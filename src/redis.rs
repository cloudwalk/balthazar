use std::ops::Deref;

use crate::*;

use bb8_redis::{bb8::Pool, RedisConnectionManager};

#[derive(Debug, Clone, Parser)]
pub struct RedisConfig {
    #[clap(long = "redis-url", env = "REDIS_URL")]
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Redis {
    pool: Pool<RedisConnectionManager>,
}

#[async_trait]
impl Feature for Redis {
    async fn init(_service_name: &str, config: EnvironmentConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.redis.url)?;
        let connection_pool = Pool::builder().build(manager).await?;

        Ok(Self {
            pool: connection_pool,
        })
    }
}

impl Deref for Redis {
    type Target = Pool<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
