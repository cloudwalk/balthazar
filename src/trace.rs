use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OtelData;
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
                //TracingFormat::Json => (Some(Layer::default().json()), None, None),
                TracingFormat::Pretty => (
                    None,
                    Some(
                        Layer::default()
                            .json()
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
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // ---------------------------------------------------------------------
        // 0 - retrieve current span and track global data
        // ---------------------------------------------------------------------
        let current_span = match ctx.lookup_current() {
            Some(span) => span,
            None => return Ok(()),
        };
        let mut current_event: Option<opentelemetry::trace::Event> = None;
        let mut current_event_thread_id: String = "".to_string();
        let mut current_event_thread_name: String = "".to_string();

        // ---------------------------------------------------------------------
        // 1 - visit context attributes
        // ---------------------------------------------------------------------
        // iterate spans aggregating all context data
        let mut log_context = serde_json::Map::default();
        for span in current_span.scope().from_root() {
            // clear current event data
            current_event = None;

            // retrieve current span data
            let span_ext = span.extensions();
            let span_data = match span_ext.get::<OtelData>() {
                Some(data) => data,
                None => continue,
            };
            let span_attrs = match &span_data.builder.attributes {
                Some(attrs) => attrs,
                None => continue,
            };

            // keep track of the last visited span data after the loop exits
            current_event = match &span_data.builder.events {
                Some(events) => events.last().map(|it| it.clone()),
                None => None,
            };

            // populate context data
            for span_attr in span_attrs {
                let key = span_attr.key.to_string();
                if key == "thread.id" {
                    current_event_thread_id = span_attr.value.to_string();
                }
                if key == "thread.name" {
                    current_event_thread_name = span_attr.value.to_string();
                }
                if CONTEXT_FIELDS_TO_IGNORE.contains(&key.as_str()) {
                    continue;
                }
                let value_wrapper = OpenTelemetryValue(span_attr.value.clone());
                log_context.insert(key, value_wrapper.into());
            }
        }

        // ---------------------------------------------------------------------
        // 2 - check has en event
        // ---------------------------------------------------------------------
        let current_event = match current_event {
            Some(event) => event,
            None => return Ok(()),
        };

        // ---------------------------------------------------------------------
        // 3 - visit event attributes
        // ---------------------------------------------------------------------
        let mut log_fields = serde_json::Map::default();
        for event in current_event.attributes {
            let key = event.key.to_string();
            if EVENT_FIELDS_TO_IGNORE.contains(&key.as_str()) {
                continue;
            }
            let value_wrapper = OpenTelemetryValue(event.value.clone());
            log_fields.insert(key.to_string(), value_wrapper.into());
        }

        // ---------------------------------------------------------------------
        // 4 - prepare log message
        // ---------------------------------------------------------------------
        let timestamp: DateTime<Utc> = current_event.timestamp.into();
        let message = LogMessage {
            level: event.metadata().level().to_string(),
            timestamp: timestamp.to_rfc3339_opts(SecondsFormat::Millis, true),
            module: event.metadata().target().to_string(),
            file: event.metadata().file().map(|it| it.to_string()),
            line: event.metadata().line(),
            span: current_span.metadata().name().to_string(),

            thread_id: current_event_thread_id.to_string(),
            thread_name: current_event_thread_name.to_string(),

            message: current_event.name.to_string(),
            fields: Value::Object(log_fields),
            context: Value::Object(log_context),
        };

        // ---------------------------------------------------------------------
        // 4 - output log message
        // ---------------------------------------------------------------------
        let message_as_json = serde_json::to_string_pretty(&message).unwrap();
        let _ = write!(writer, "{}\n", message_as_json);

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct LogMessage {
    level: String,
    timestamp: String,

    module: String,
    file: Option<String>,
    line: Option<u32>,
    span: String,

    thread_id: String,
    thread_name: String,

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
