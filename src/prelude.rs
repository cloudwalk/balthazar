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
pub use tokio::{main as async_main, sync, task, time};

pub use tracing::{debug, error, info, instrument, instrument::Instrument, span, trace, warn};

#[cfg(feature = "database")]
pub use sqlx as sql;

// Feature enablement
pub trait Feature {
    fn init(config: EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}
