use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;
use std::thread;
use std::time::SystemTime;
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_serde::fields::AsMap;
use tracing_subscriber::fmt::format::{DefaultFields, Writer};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, Layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use crate::{async_trait, EnvironmentConfig, Feature, Parser, Result};

// -----------------------------------------------------------------------------
// Supported Formats
// -----------------------------------------------------------------------------
#[derive(clap::ArgEnum, Clone, Debug)]
pub enum TracingFormat {
    None,
    Hierarchical,
    Pretty,
    Json,
}

// -----------------------------------------------------------------------------
// Config
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, Parser)]
pub struct TracingConfig {
    #[clap(
        long = "tracing-disable-opentelemetry",
        env = "TRACING_DISABLE_OPENTELEMETRY"
    )]
    pub disable_opentelemetry: bool,

    #[clap(
        long = "tracing-opentelemetry-endpoint",
        env = "TRACING_OPENTELEMETRY_ENDPOINT",
        default_value = "http://localhost:14268/api/traces"
    )]
    pub opentelemetry_endpoint: String,

    #[clap(
        long = "tracing-log-level",
        env = "TRACING_LOG_LEVEL",
        default_value = "debug"
    )]
    pub log_level: String,

    #[clap(
        arg_enum,
        long = "tracing-format",
        env = "TRACING_FORMAT",
        default_value = "pretty"
    )]
    pub format: TracingFormat,
}

// -----------------------------------------------------------------------------
// Service
// -----------------------------------------------------------------------------
#[derive(clap::Parser, Debug, Clone)]
pub struct Tracing;

#[async_trait]
impl Feature for Tracing {
    async fn init(service_name: &str, config: EnvironmentConfig) -> Result<Self> {
        std::env::set_var("RUST_LOG", &config.tracing.log_level);

        let telemetry = if config.tracing.disable_opentelemetry {
            None
        } else {
            let tracer = opentelemetry_jaeger::new_pipeline()
                .with_collector_endpoint(&config.tracing.opentelemetry_endpoint)
                .with_service_name(service_name)
                .install_batch(opentelemetry::runtime::Tokio)?;
            Some(
                tracing_opentelemetry::layer()
                    .with_tracked_inactivity(false)
                    .with_tracer(tracer),
            )
        };

        // tracing_subscriber lib currently does not support dynamically adding layer to registry
        // accordingly to some condition. this can be verified in the following issues:
        // https://github.com/tokio-rs/tracing/issues/575
        // https://github.com/tokio-rs/tracing/issues/1708
        //
        // but there is a workaround described here:
        // https://github.com/tokio-rs/tracing/issues/894
        //
        // the workaround consists of passing a optional of Layer to every conditional layer,
        // so if Some(layer) is passed, that layer is active, if None the layer is inactive.
        let (layer_format_json, layer_format_pretty, layer_format_hierarchical) =
            match config.tracing.format {
                TracingFormat::None => (None, None, None),
                TracingFormat::Json => (
                    Some(Layer::default().event_format(JsonFormatter)),
                    None,
                    None,
                ),
                TracingFormat::Pretty => (
                    None,
                    Some(
                        Layer::default()
                            .pretty()
                            .with_thread_ids(true)
                            .with_thread_names(true)
                            .with_target(true)
                            .with_file(true)
                            .with_line_number(true)
                            .with_ansi(!config.core.no_color),
                    ),
                    None,
                ),
                TracingFormat::Hierarchical => (
                    None,
                    None,
                    Some(
                        HierarchicalLayer::new(2)
                            .with_targets(true)
                            .with_bracketed_fields(true)
                            .with_ansi(!config.core.no_color),
                    ),
                ),
            };

        Registry::default()
            .with(EnvFilter::from_default_env())
            .with(telemetry)
            .with(layer_format_json)
            .with(layer_format_pretty)
            .with(layer_format_hierarchical)
            .init();

        tracing::debug!("started tracer");

        Ok(Self)
    }
}

impl Drop for Tracing {
    fn drop(&mut self) {
        tracing::debug!("stopping tracer");
        opentelemetry::global::shutdown_tracer_provider();
    }
}

// -----------------------------------------------------------------------------
// Json Formatter
// -----------------------------------------------------------------------------
const CONTEXT_FIELDS_TO_IGNORE: [&str; 5] = [
    "code.filepath",
    "code.lineno",
    "code.namespace",
    "thread.id",
    "thread.name",
];
const EVENT_FIELDS_TO_IGNORE: [&str; 6] = [
    "code.filepath",
    "code.lineno",
    "code.namespace",
    "level",
    "name",
    "target",
];

struct JsonFormatter;

impl<S> FormatEvent<S, DefaultFields> for JsonFormatter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, DefaultFields>,
        writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // ---------------------------------------------------------------------
        // 0 - retrieve current span and track global data
        // ---------------------------------------------------------------------
        let current_span = match ctx.lookup_current() {
            Some(span) => span,
            None => {
                log_without_context(writer, event);
                return Ok(());
            }
        };
        let mut ot_event: Option<opentelemetry::trace::Event> = None;
        let mut field_thread_id: String = "".to_string();
        let mut field_thread_name: String = "".to_string();

        // ---------------------------------------------------------------------
        // 1 - visit context attributes
        // ---------------------------------------------------------------------
        // iterate spans aggregating all context data from several spans in a single map
        // iteration is performed from lower level span (current) to higher level span (root)
        let mut is_first_span = true;
        let mut field_context = serde_json::Map::default();
        for span in current_span.scope() {
            // 1.1 - retrieve span data
            let span_ext = span.extensions();
            let span_data = match span_ext.get::<OtelData>() {
                Some(data) => data,
                None => {
                    is_first_span = false;
                    continue;
                }
            };

            // 1.2 - keep track of current event for use after the iteration
            if is_first_span {
                ot_event = match &span_data.builder.events {
                    Some(events) => events.last().cloned(),
                    None => None,
                };
            }

            // 1.3 - retrieve span attributes
            let span_attrs = match &span_data.builder.attributes {
                Some(attrs) => attrs,
                None => {
                    is_first_span = false;
                    continue;
                }
            };

            // 1.4 - populate context data
            for span_attr in span_attrs {
                // parse key
                let key = span_attr.key.to_string();

                // track thread
                if is_first_span && key == "thread.id" {
                    field_thread_id = span_attr.value.to_string();
                }
                if is_first_span && key == "thread.name" {
                    field_thread_name = span_attr.value.to_string();
                }

                // check ignored fields
                if CONTEXT_FIELDS_TO_IGNORE.contains(&key.as_str()) {
                    continue;
                }

                // add attr to context if not already present because lower level attrs
                // have precedence over higher level attrs if they have the same name
                let value_wrapper = OpenTelemetryValue(span_attr.value.clone());
                if !field_context.contains_key(&key) {
                    field_context.insert(key, value_wrapper.into());
                }
            }
            is_first_span = false;
        }

        // ---------------------------------------------------------------------
        // 2 - ensure has en event
        // ---------------------------------------------------------------------
        let ot_event = match ot_event {
            Some(event) => event,
            None => {
                // unlikely to happen because if it has a span, it will have an event because this
                // function is called only when event exists
                log_without_context(writer, event);
                return Ok(());
            }
        };

        // ---------------------------------------------------------------------
        // 3 - visit event attributes
        // ---------------------------------------------------------------------
        let mut field_fields = serde_json::Map::default();
        for event in &ot_event.attributes {
            // parse key
            let key = event.key.to_string();

            // check ignored fields
            if EVENT_FIELDS_TO_IGNORE.contains(&key.as_str()) {
                continue;
            }

            // add event attr to context
            let value_wrapper = OpenTelemetryValue(event.value.clone());
            field_fields.insert(key.to_string(), value_wrapper.into());
        }

        // ---------------------------------------------------------------------
        // 4 - output log message
        // ---------------------------------------------------------------------
        log_with_context(
            writer,
            event,
            ot_event,
            field_context,
            field_fields,
            current_span.name().into(),
            field_thread_id,
            field_thread_name,
        );
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct LogMessage {
    level: String,
    timestamp: String,

    target: String,
    file: Option<String>,
    line: Option<u32>,
    span: String,

    thread_id: String,
    thread_name: String,

    message: String,
    fields: Value,

    context: Value,
}

fn log_without_context(writer: Writer<'_>, event: &Event) {
    // extract fields from event
    let event_fields = event.field_map();
    let mut event_fields_as_value = match serde_json::to_value(&event_fields) {
        Ok(v) => v,
        Err(_) => return, // unlikely to happen, so just ignore it for now
    };

    // parse fields from alternative sources
    let field_timestamp: DateTime<Utc> = SystemTime::now().into();
    let field_thread_name = thread::current()
        .name()
        .map(|it| it.to_string())
        .unwrap_or_default();
    let field_message = event_fields_as_value
        .get("message")
        .and_then(|it| it.as_str())
        .map(|it| it.to_string())
        .unwrap_or_default();

    // remove message from event fields because it is special and will go to a separe attribute
    if let Some(obj) = event_fields_as_value.as_object_mut() {
        obj.remove("message");
    }

    let message = LogMessage {
        level: event.metadata().level().to_string(),
        timestamp: field_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        target: event.metadata().target().to_string(),
        file: event.metadata().file().map(|it| it.to_string()),
        line: event.metadata().line(),
        span: "".to_string(),

        thread_id: "".to_string(),
        thread_name: field_thread_name,

        message: field_message,
        fields: Value::Object(
            event_fields_as_value
                .as_object()
                .cloned()
                .unwrap_or_default(),
        ),
        context: Value::Object(serde_json::Map::default()),
    };

    log(writer, message);
}

#[allow(clippy::too_many_arguments)]
fn log_with_context(
    writer: Writer<'_>,
    event: &Event,
    ot_event: opentelemetry::trace::Event,
    field_context: serde_json::Map<String, Value>,
    field_fields: serde_json::Map<String, Value>,
    field_span: String,
    field_thread_id: String,
    field_thread_name: String,
) {
    let field_timestamp: DateTime<Utc> = ot_event.timestamp.into();
    let message = LogMessage {
        level: event.metadata().level().to_string(),
        timestamp: field_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
        target: event.metadata().target().to_string(),
        file: event.metadata().file().map(|it| it.to_string()),
        line: event.metadata().line(),
        span: field_span,

        thread_id: field_thread_id,
        thread_name: field_thread_name,

        message: ot_event.name.to_string(),
        fields: Value::Object(field_fields),
        context: Value::Object(field_context),
    };

    log(writer, message);
}

fn log(mut writer: Writer<'_>, message: LogMessage) {
    if let Ok(message_as_json) = serde_json::to_string(&message) {
        let _ = write!(writer, "{}\n", message_as_json);
    }
}

// -----------------------------------------------------------------------------
// Converters
// -----------------------------------------------------------------------------
struct OpenTelemetryValue(opentelemetry::Value);

impl From<OpenTelemetryValue> for serde_json::Value {
    fn from(value: OpenTelemetryValue) -> Self {
        match &value.0 {
            opentelemetry::Value::Bool(v) => Value::Bool(*v),
            opentelemetry::Value::I64(v) => Value::from(*v),
            opentelemetry::Value::F64(v) => Value::from(*v),
            opentelemetry::Value::String(v) => Value::String(v.to_string()),
            opentelemetry::Value::Array(_) => Value::String("".to_string()),
        }
    }
}
