use std::ops::Deref;

use sqlx::{postgres::PgPoolOptions, Pool};
use sqlx::Postgres as LibPosgtres;

use crate::*;

#[derive(Debug, Clone, Parser)]
pub struct PostgresConfig {
    #[clap(
        long = "postgres-url",
        env = "POSTGRES_URL"
    )]
    pub url: String,

    #[clap(
        long = "postgres-max-connections",
        env = "POSTGRES_MAX_CONNECTIONS",
        default_value = "8"
    )]
    pub pool_max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct Postgres {
    pool: Pool<LibPosgtres>,
}

#[crate::async_trait]
impl Feature for Postgres {
    async fn init(_service_name: &str, config: EnvironmentConfig) -> Result<Self> {
        Ok(Self {
            pool: PgPoolOptions::new()
                .max_connections(config.postgres.pool_max_connections)
                .connect(&config.postgres.url)
                .await?,
        })
    }
}

impl Deref for Postgres {
    type Target = Pool<LibPosgtres>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
