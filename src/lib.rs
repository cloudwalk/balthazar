use std::fmt::Debug;

pub mod build_info;
mod core;
mod lang;
mod trace;

pub use crate::core::CoreConfig;
#[allow(deprecated)]
pub use crate::lang::sensitive::{Sensitive, SensitiveString};
pub use crate::trace::{Tracing, TracingConfig, TracingFormat};

pub use async_trait::async_trait;
pub use clap::{self, Args, Parser};
pub use ethereum_types::{H256, U256};
pub use eyre::Result;
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

// Feature enablement
#[async_trait]
pub trait Feature {
    async fn init(service_name: &str, config: EnvironmentConfig) -> Result<Self>
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

        core::Core::init(service_name.as_ref(), environment.clone()).await?;

        Ok(Environment {
            service_name: service_name.as_ref().to_string(),
            tracing: Tracing::init(service_name.as_ref(), environment.clone()).await?,

            #[cfg(feature = "postgres")]
            postgres: Postgres::init(service_name.as_ref(), environment.clone()).await?,

            #[cfg(feature = "redis")]
            redis: Redis::init(service_name.as_ref(), environment.clone()).await?,

            config: Self {
                project,
                environment,
            },
        })
    }
}
