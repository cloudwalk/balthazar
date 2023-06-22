use std::ops::Deref;

use crate::*;

use bb8_redis::{bb8::Pool, RedisMultiplexedConnectionManager};

#[derive(Debug, Clone, Parser)]
pub struct RedisConfig {
    #[clap(long = "redis-url", env = "REDIS_URL")]
    pub url: Sensitive<String>,
}

#[derive(Debug, Clone)]
pub struct Redis {
    pool: Pool<RedisMultiplexedConnectionManager>,
}

#[async_trait]
impl Feature for Redis {
    async fn init(_service_name: &str, config: &EnvironmentConfig) -> Result<Self> {
        let manager = RedisMultiplexedConnectionManager::new(config.redis.url.0.clone())?;
        let connection_pool = Pool::builder().build(manager).await?;

        Ok(Self {
            pool: connection_pool,
        })
    }
}

impl Deref for Redis {
    type Target = Pool<RedisMultiplexedConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}
