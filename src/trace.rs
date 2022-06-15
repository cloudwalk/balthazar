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
use tracing_subscriber::registry::{LookupSpan, SpanRef};
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
    JsonPretty,
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
                    Some(
                        Layer::default()
                            .event_format(JsonFormatter::new(service_name.to_string(), false)),
                    ),
                    None,
                    None,
                ),
                TracingFormat::JsonPretty => (
                    Some(
                        Layer::default()
                            .event_format(JsonFormatter::new(service_name.to_string(), true)),
                    ),
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

struct JsonFormatter {
    service_name: String,
    pretty: bool,
}

impl JsonFormatter {
    fn new(service_name: String, pretty: bool) -> Self {
        Self {
            service_name,
            pretty,
        }
    }

    fn parse_from_service(&self, target: &str) -> u8 {
        match target.to_string().starts_with(&self.service_name) {
            true => 1,
            false => 0
        }
    }

    fn parse_simple_name(&self, name: String) -> String {
        name.split("::")
            .map(|it| it.to_string())
            .last()
            .unwrap_or_else(|| name.clone())
    }

    fn log_without_context(&self, writer: Writer<'_>, event: &Event) {
        // extract fields from event
        let event_fields = event.field_map();
        let mut event_fields_as_value = match serde_json::to_value(&event_fields) {
            Ok(v) => v,
            Err(_) => return, // unlikely to happen, so just ignore it for now
        };

        // parse fields from alternative sources
        let field_timestamp: DateTime<Utc> = SystemTime::now().into();

        let field_target = event.metadata().target().to_string();

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

        let log_message = LogMessage {
            from_service: self.parse_from_service(&field_target),

            level: event.metadata().level().to_string(),
            timestamp: field_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),

            target: field_target.clone(),
            target_simple: self.parse_simple_name(field_target),

            file: event.metadata().file().map(|it| it.to_string()),
            line: event.metadata().line(),

            thread_id: "".to_string(),
            thread_name: field_thread_name,

            current_span_simple: "".to_string(),
            current_span: "".to_string(),
            current_span_id: 0,

            root_span_simple: "".to_string(),
            root_span: "".to_string(),
            root_span_id: 0,

            message: field_message,
            fields: Value::Object(
                event_fields_as_value
                    .as_object()
                    .cloned()
                    .unwrap_or_default(),
            ),
            context: Value::Object(serde_json::Map::default()),
        };

        self.log(writer, log_message);
    }

    #[allow(clippy::too_many_arguments)]
    fn log_with_context(
        &self,
        writer: Writer<'_>,
        event: &Event,
        ot_event: opentelemetry::trace::Event,
        field_context: serde_json::Map<String, Value>,
        field_fields: serde_json::Map<String, Value>,
        field_root_span_id: u64,
        field_root_span_name: String,
        field_current_span_id: u64,
        field_current_span_name: String,
        field_thread_id: String,
        field_thread_name: String,
    ) {
        let field_timestamp: DateTime<Utc> = ot_event.timestamp.into();
        let field_target = event.metadata().target().to_string();
        let message = LogMessage {
            from_service: self.parse_from_service(&field_target),

            level: event.metadata().level().to_string(),
            timestamp: field_timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),

            target: field_target.clone(),
            target_simple: self.parse_simple_name(field_target),

            file: event.metadata().file().map(|it| it.to_string()),
            line: event.metadata().line(),

            thread_id: field_thread_id,
            thread_name: field_thread_name,

            root_span: field_root_span_name.clone(),
            root_span_simple: self.parse_simple_name(field_root_span_name),
            root_span_id: field_root_span_id,

            current_span: field_current_span_name.clone(),
            current_span_simple: self.parse_simple_name(field_current_span_name),
            current_span_id: field_current_span_id,

            message: ot_event.name.to_string(),
            fields: Value::Object(field_fields),
            context: Value::Object(field_context),
        };

        self.log(writer, message);
    }

    fn log(&self, mut writer: Writer<'_>, message: LogMessage) {
        let message_as_json_result = if self.pretty {
            serde_json::to_string_pretty(&message)
        } else {
            serde_json::to_string(&message)
        };
        if let Ok(message_as_json) = message_as_json_result {
            let _ = writeln!(writer, "{}", message_as_json);
        }
    }
}

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
                self.log_without_context(writer, event);
                return Ok(());
            }
        };

        let mut ot_event: Option<opentelemetry::trace::Event> = None;
        let mut field_root_span_id = 0u64;
        let mut field_root_span_name = "".to_string();
        let mut field_thread_id = "".to_string();
        let mut field_thread_name = "".to_string();

        // ---------------------------------------------------------------------
        // 1 - visit context attributes
        // ---------------------------------------------------------------------
        // iterate spans aggregating all context data from all spans into a single map
        let mut field_context = serde_json::Map::default();

        let spans: Vec<SpanRef<S>> = current_span.scope().from_root().collect();
        for (index, span) in spans.iter().enumerate() {
            let is_root_span = index == 0;
            let is_current_span = index == spans.len() - 1;

            // 1.1 - retrieve span data
            let span_ext = span.extensions();
            let span_data = match span_ext.get::<OtelData>() {
                Some(data) => data,
                None => continue,
            };

            // 1.2 - keep track of current OpenTracing event for use after iteration
            if is_current_span {
                ot_event = match &span_data.builder.events {
                    Some(events) => events.last().cloned(),
                    None => None,
                };
            }

            // 1.3 - keep track of root span name for use after iteration
            if is_root_span {
                field_root_span_id = span.id().into_u64();
                field_root_span_name = span.name().to_string();
            }

            // 1.4 - retrieve span attributes
            let span_attrs = match &span_data.builder.attributes {
                Some(attrs) => attrs,
                None => continue,
            };

            // 1.5 - populate context data
            for span_attr in span_attrs {
                // parse key
                let key = span_attr.key.to_string();

                // track thread
                if is_current_span && key == "thread.id" {
                    field_thread_id = span_attr.value.to_string();
                }
                if is_current_span && key == "thread.name" {
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
        }

        // ---------------------------------------------------------------------
        // 2 - ensure has en event
        // ---------------------------------------------------------------------
        let ot_event = match ot_event {
            Some(event) => event,
            None => {
                // unlikely to happen because if it has a span, it will have an event because this
                // function is called only when event exists
                self.log_without_context(writer, event);
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
        self.log_with_context(
            writer,
            event,
            ot_event,
            field_context,
            field_fields,
            field_root_span_id,
            field_root_span_name,
            current_span.id().into_u64(),
            current_span.name().into(),
            field_thread_id,
            field_thread_name,
        );
        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct LogMessage {
    from_service: u8,

    level: String,
    timestamp: String,

    target_simple: String,
    target: String,
    file: Option<String>,
    line: Option<u32>,

    thread_id: String,
    thread_name: String,

    root_span_simple: String,
    root_span: String,
    root_span_id: u64,

    current_span_simple: String,
    current_span: String,
    current_span_id: u64,

    message: String,
    fields: Value,
    context: Value,
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
