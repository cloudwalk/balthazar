use std::fmt::Debug;

use clap::{Args, Parser};
use eyre::Result;

mod database;
mod tracing;

pub use self::database::*;
pub use self::tracing::*;

#[derive(Debug, Clone)]
pub struct Environment<T: Debug + Clone + Args> {
    pub config: Config<T>,
    pub tracing: Tracing,
    pub database: Database,
}

#[derive(Debug, Clone, Parser)]
pub struct EnvironmentConfig {
    #[clap(flatten)]
    pub database: DatabaseConfig,
    #[clap(flatten)]
    pub tracing: TracingConfig,
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

        Ok(Environment {
            tracing: Tracing::init(environment.tracing.clone())?,
            database: Database::init(environment.database.clone()).await?,
            config: Self {
                project,
                environment,
            },
        })
    }
}
