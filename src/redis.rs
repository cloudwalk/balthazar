use std::ops::Deref;

use crate::*;

use bb8_redis::{
    bb8::Pool,
    RedisConnectionManager,
};

#[derive(Debug, Clone, Parser)]
pub struct RedisConfig {
    #[clap(
        env = "REDIS_CONNECTION_STRING",
        default_value = "redis://default:password@127.0.0.1:6379"
    )]
    pub connection_string: String,
}

#[derive(Debug, Clone)]
pub struct Redis {
    pool: Pool<RedisConnectionManager>,
}

#[async_trait]
impl Feature for Redis {
    async fn init(_service_name: &str, config: EnvironmentConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.redis.connection_string)?;
        let connection_pool = Pool::builder().build(manager).await?;

        Ok(Self { pool: connection_pool })
    }
}

impl Deref for Redis {
    type Target = Pool<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.pool                                    
    }                                                        
} 
