use std::ops::Deref;

use clap::Parser;
use eyre::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub use sqlx;

#[derive(Debug, Clone, Parser)]
pub struct DatabaseConfig {
    #[clap(env = "DATABASE_URL")]
    pub url: String,

    #[clap(env = "DATABASE_POOL_MAX_CONNECTIONS", default_value = "8")]
    pub pool_max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn init(config: DatabaseConfig) -> Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .max_connections(config.pool_max_connections)
                .connect(&config.url)
                .await?,
        })
    }
}

impl Deref for Database {
    type Target = Pool<Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
