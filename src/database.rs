use std::ops::Deref;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::*;

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
    pub async fn init(_service_name: String, config: EnvironmentConfig) -> Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .max_connections(config.database.pool_max_connections)
                .connect(&config.database.url)
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
