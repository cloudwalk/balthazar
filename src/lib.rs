use std::fmt::Debug;

mod core;

#[cfg(logging)]
mod tracing;

#[cfg(database)]
mod database;

pub mod prelude;

use prelude::*;

#[derive(Debug, Clone)]
pub struct Environment<T: Debug + Clone + Args> {
    pub config: Config<T>,

    #[cfg(logging)]
    pub tracing: Tracing,
    #[cfg(database)]
    pub database: Database,
}

#[derive(Debug, Clone, Parser)]
pub struct EnvironmentConfig {
    #[clap(flatten)]
    pub core: core::CoreConfig,

    #[cfg(database)]
    #[clap(flatten)]
    pub database: DatabaseConfig,

    #[cfg(logging)]
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

        core::Core::init(environment.clone())?;

        Ok(Environment {
            #[cfg(logging)]
            tracing: Tracing::init(environment.clone())?,

            #[cfg(database)]
            database: Database::init(environment.clone()).await?,

            config: Self {
                project,
                environment,
            },
        })
    }
}
