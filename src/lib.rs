use std::fmt::Debug;
mod core;
mod tracing;
pub use crate::{
    tracing::{Tracing, TracingConfig},
    Environment, EnvironmentConfig,
};
pub use clap::{self, Args, Parser};
pub use eyre::Result;
pub use thiserror::Error;
pub use tokio::{main, sync, task, time};
pub use tracing::{debug, error, info, span, trace, warn};

#[cfg(feature = "database")]
mod database;
#[cfg(feature = "database")]
pub use crate::database::{Database, DatabaseConfig};
#[cfg(feature = "database")]
pub use sqlx as sql;

// Feature enablement
pub trait Feature {
    fn init(config: EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct Environment<T: Debug + Clone + Args> {
    pub config: Config<T>,
    pub tracing: tracing::Tracing,

    #[cfg(feature = "database")]
    pub database: Database,
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
}

#[derive(Debug, Clone, Parser)]
pub struct Config<T: Debug + Clone + Args> {
    #[clap(flatten)]
    pub project: T,

    #[clap(flatten)]
    pub environment: EnvironmentConfig,
}

impl<T: Debug + Clone + Args> Config<T> {
    pub async fn init() -> Result<Environment<T>> {
        let Self {
            project,
            environment,
        } = Self::parse();

        core::Core::init(environment.clone())?;

        Ok(Environment {
            tracing: Tracing::init(environment.clone())?,

            #[cfg(feature = "database")]
            database: Database::init(environment.clone()).await?,

            config: Self {
                project,
                environment,
            },
        })
    }
}
