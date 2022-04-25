use std::fmt::Debug;

mod lang;
mod core;
mod tracing;

pub use crate::lang::sensitive_string::SensitiveString;
pub use crate::tracing::{Tracing, TracingConfig};

pub use async_trait::async_trait;
pub use clap::{self, Args, Parser};
pub use ethereum_types::{H256, U256};
pub use eyre::Result;
pub use futures_util::StreamExt;
pub use tokio::{self, main};

#[cfg(feature = "database")]
mod database;

#[cfg(feature = "database")]
pub use crate::database::{Database, DatabaseConfig};

#[cfg(feature = "redis")]
mod redis;

#[cfg(feature = "redis")]
pub use crate::redis::{Redis, RedisConfig};

// Feature enablement
pub trait Feature {
    fn init(service_name: String, config: EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Environment<T: Debug + Clone + Args> {
    pub service_name: String,
    pub config: Config<T>,
    pub tracing: tracing::Tracing,

    #[cfg(feature = "database")]
    pub database: Database,

    #[cfg(feature = "redis")]
    pub redis: Redis,
}

#[derive(Debug, Clone, Parser)]
pub struct EnvironmentConfig {
    #[clap(flatten)]
    pub core: core::CoreConfig,

    #[clap(flatten)]
    pub tracing: tracing::TracingConfig,

    #[cfg(feature = "database")]
    #[clap(flatten)]
    pub database: DatabaseConfig,

    #[cfg(feature = "redis")]
    #[clap(flatten)]
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Parser)]
pub struct Config<T: Debug + Clone + Args> {
    #[clap(flatten)]
    pub project: T,

    #[clap(flatten)]
    pub environment: EnvironmentConfig,
}

impl<T: Debug + Clone + Args> Config<T> {
    pub async fn init(service_name: String) -> Result<Environment<T>> {
        let Self {
            project,
            environment,
        } = Self::parse();

        core::Core::init(service_name.clone(), environment.clone())?;

        Ok(Environment {
            service_name: service_name.clone(),
            tracing: Tracing::init(service_name.clone(), environment.clone())?,

            #[cfg(feature = "database")]
            database: Database::init(service_name.clone(), environment.clone()).await?,

            #[cfg(feature = "redis")]
            redis: Redis::init(service_name.clone(), environment.clone()).await?,

            config: Self {
                project,
                environment,
            },
        })
    }
}
