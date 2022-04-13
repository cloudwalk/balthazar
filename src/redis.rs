use crate::*;

use bb8_redis::{
    bb8::{self, Pool, RunError},
    redis::AsyncCommands,
    RedisConnectionManager,
};

use std::error::Error;

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
    pool: bb8::Pool<RedisConnectionManager>,
}

impl Redis {
    pub async fn init(_service_name: String, config: EnvironmentConfig) -> Result<Self> {
        let manager = RedisConnectionManager::new(config.redis.connection_string)?;
        let connection_pool = bb8::Pool::builder().build(manager).await?;

        Ok(Self { pool: connection_pool })
    }
}

impl Deref for Redis {
    type Target = bb8::Pool<RedisConnectionManager>;

    fn deref(&self) -> &Self::Target {
        &self.pool                                    
    }                                                        
} 
