use std::fmt::Debug;

pub mod build_info;
mod core;
pub mod health_status;
mod lang;
mod timeable;
mod trace;

pub use crate::core::CoreConfig;
#[allow(deprecated)]
pub use crate::lang::sensitive::{Sensitive, SensitiveString};
pub use crate::trace::{
    HoneycombConfig, MakeSpanWithContext, RequestTracerPropagation, Tracing, TracingConfig,
    TracingFormat, UuidMakeRequestId,
};

pub use async_trait::async_trait;
pub use clap::{self, Args, Parser};
pub use ethereum_types::{H128, H160, H256, H264, H32, H512, H520, H64, U128, U256, U512, U64};
pub use eyre::{eyre as throw, Result};
pub use futures_util::StreamExt;
pub use quickcheck::Arbitrary;
pub use serde::{self, Deserialize, Serialize};
pub use strum;
pub use time;
pub use tokio::{self, main, test};
pub use tracing::{self, debug, error, info, warn};
pub use uuid::Uuid;

#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use crate::postgres::{Postgres, PostgresConfig};

#[cfg(feature = "redis")]
mod redis;

#[cfg(feature = "redis")]
pub use crate::redis::{Redis, RedisConfig};

#[cfg(feature = "streaming")]
mod streaming;
#[cfg(feature = "streaming")]
pub use streaming::{KafkaClient, KafkaConfig, Message, StreamingClient};

pub use timeable::Timeable;

// Feature enablement
#[async_trait]
pub trait Feature {
    async fn init(service_name: &str, config: &EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct Environment<T: Debug + Args> {
    pub service_name: String,
    pub config: Config<T>,
    pub tracing: Tracing,

    #[cfg(feature = "postgres")]
    pub postgres: Postgres,

    #[cfg(feature = "redis")]
    pub redis: Redis,

    #[cfg(feature = "streaming")]
    pub kafka: KafkaClient,
}

#[derive(Debug, Clone, Parser)]
pub struct EnvironmentConfig {
    #[clap(flatten)]
    pub core: core::CoreConfig,

    #[clap(flatten)]
    pub tracing: TracingConfig,

    #[cfg(feature = "postgres")]
    #[clap(flatten)]
    pub postgres: PostgresConfig,

    #[cfg(feature = "redis")]
    #[clap(flatten)]
    pub redis: RedisConfig,

    #[cfg(feature = "streaming")]
    #[clap(flatten)]
    pub kafka: KafkaConfig,
}

#[derive(Debug, Parser)]
pub struct Config<T: Debug + Args> {
    #[clap(flatten)]
    pub project: T,

    #[clap(flatten)]
    pub environment: EnvironmentConfig,
}

impl<T: Debug + Args> Config<T> {
    pub async fn init<S: AsRef<str>>(service_name: S) -> Result<Environment<T>> {
        let Self {
            project,
            environment,
        } = Self::parse();

        core::Core::init(service_name.as_ref(), &environment).await?;
        timeable::init(service_name.as_ref());

        Ok(Environment {
            service_name: service_name.as_ref().to_string(),
            tracing: Tracing::init(service_name.as_ref(), &environment).await?,

            #[cfg(feature = "postgres")]
            postgres: Postgres::init(service_name.as_ref(), &environment).await?,

            #[cfg(feature = "redis")]
            redis: Redis::init(service_name.as_ref(), &environment).await?,

            #[cfg(feature = "streaming")]
            kafka: KafkaClient::new(&environment.kafka).await?,

            config: Self {
                project,
                environment,
            },
        })
    }
}
