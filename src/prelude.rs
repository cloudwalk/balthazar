pub use crate::{Environment, EnvironmentConfig};

// Traits
pub use clap::{self, Args, Parser};
pub use thiserror::Error;

// Type Replacements
pub use eyre::Result;
pub use tokio::{main as async_main, sync::RwLock};

#[cfg(logging)]
pub use tracing::{debug, error, info, instrument, span, trace, warn};

#[cfg(database)]
pub use sqlx as sql;

// Feature enablement
pub trait Feature {
    fn init(config: EnvironmentConfig) -> Result<Self>
    where
        Self: Sized;
}
