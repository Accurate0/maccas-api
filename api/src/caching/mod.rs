use deadpool_redis::Runtime;
use prost::Message;
use prost::bytes::Bytes;
use redis::{AsyncCommands, ToRedisArgs};

pub use prost_types::Timestamp as ProtobufTimestamp;
use tracing::instrument;

pub mod protos {
    include!(concat!(env!("OUT_DIR"), "/maccas.caching.rs"));
}

pub struct Redis {
    pool: deadpool_redis::Pool,
}

#[derive(thiserror::Error, Debug)]
pub enum RedisError {
    #[error(transparent)]
    ConnectionPoolError(#[from] deadpool_redis::CreatePoolError),
    #[error(transparent)]
    PoolError(#[from] deadpool_redis::PoolError),
    #[error(transparent)]
    RedisError(#[from] redis::RedisError),
}

impl Clone for Redis {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

impl Redis {
    pub async fn new(connection_string: &str) -> Result<Self, RedisError> {
        let pool = deadpool_redis::Config::from_url(connection_string)
            .create_pool(Some(Runtime::Tokio1))?;

        Ok(Self { pool })
    }

    pub async fn set_ex<K, V>(&self, k: K, v: V) -> Result<(), RedisError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.pool.get().await?;
        conn.set_ex(k, v, 86400).await.map_err(RedisError::from)
    }

    pub async fn mget<K>(&self, k: K) -> Result<Vec<Option<Bytes>>, RedisError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.pool.get().await?;
        conn.mget(k).await.map_err(RedisError::from)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OfferDetailsCacheError {
    #[error(transparent)]
    RedisError(#[from] RedisError),
}

impl Clone for OfferDetailsCache {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
        }
    }
}

pub struct OfferDetailsCache {
    redis: Redis,
}

impl OfferDetailsCache {
    const PREFIX: &str = "maccas:offer_details";

    pub fn new(redis: Redis) -> Self {
        Self { redis }
    }

    #[instrument(name = "OfferDetailsCache::set", skip(self, details))]
    pub async fn set(&self, details: protos::OfferDetails) -> Result<(), OfferDetailsCacheError> {
        let bytes = details.encode_to_vec();
        let _ = self
            .redis
            .set_ex(
                format!("{}:{}", Self::PREFIX, details.proposition_id),
                bytes,
            )
            .await;

        Ok(())
    }

    #[instrument(name = "OfferDetailsCache::get_all", skip(self, ids))]
    pub async fn get_all(
        &self,
        ids: &[i64],
    ) -> Result<Vec<Option<protos::OfferDetails>>, OfferDetailsCacheError> {
        Ok(self
            .redis
            .mget(
                ids.iter()
                    .map(|id| format!("{}:{}", Self::PREFIX, id))
                    .collect::<Vec<_>>(),
            )
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|b| {
                if let Some(b) = b {
                    if let Ok(r) = Message::decode(b) {
                        Some(r)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect())
    }
}
