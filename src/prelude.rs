pub use crate::{
    tracing::{Tracing, TracingConfig},
    Environment, EnvironmentConfig,
};

#[cfg(feature = "database")]
pub use crate::database::{Database, DatabaseConfig};

// Traits
pub use clap::{self, Args, Parser};
pub use thiserror::Error;

// Type Replacements
pub use eyre::Result;
pub use tokio::{self, main};

#[cfg(feature = "database")]
pub use sqlx as sql;

// Feature enablement
pub trait Feature {
    fn init(config: EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}
