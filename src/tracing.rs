use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

use crate::{EnvironmentConfig, Feature, Parser, Result};

#[derive(Debug, Clone, Parser)]
pub struct TracingConfig {
    #[clap(env = "TRACING_DISABLE_OPENTELEMETRY", parse(try_from_str))]
    pub disable_opentelemetry: bool,

    #[clap(
        env = "TRACING_OPENTELEMETRY_ENDPOINT",
        default_value = "http://localhost:14268/api/traces"
    )]
    pub opentelemetry_endpoint: String,

    #[clap(env = "TRACING_LOG_LEVEL", default_value = "debug")]
    pub log_level: String,
}

#[derive(Debug, Clone)]
pub struct Tracing;

impl Feature for Tracing {
    fn init(config: EnvironmentConfig) -> Result<Self> {
        std::env::set_var("RUST_LOG", &config.tracing.log_level);

        let registry = Registry::default()
            .with(EnvFilter::from_default_env())
            .with(
                HierarchicalLayer::new(2)
                    .with_targets(true)
                    .with_bracketed_fields(true)
                    .with_ansi(config.core.no_color),
            );

        if config.tracing.disable_opentelemetry {
            registry.init();
        } else {
            let tracer = opentelemetry_jaeger::new_pipeline()
                .with_collector_endpoint(&config.tracing.opentelemetry_endpoint)
                .with_service_name(module_path!())
                .install_batch(opentelemetry::runtime::Tokio)?;

            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

            registry.with(telemetry).init();
        }

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
