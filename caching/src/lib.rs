use deadpool_redis::Runtime;
use prost::Message;
use prost::bytes::Bytes;
use redis::{AsyncCommands, ToRedisArgs};

pub use prost_types::Timestamp as ProtobufTimestamp;

pub mod maccas {
    pub mod caching {
        include!(concat!(env!("OUT_DIR"), "/maccas.caching.rs"));
    }
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

impl Redis {
    pub async fn new(connection_string: &str) -> Result<Self, RedisError> {
        let pool = deadpool_redis::Config::from_url(connection_string)
            .create_pool(Some(Runtime::Tokio1))?;

        Ok(Self { pool })
    }

    pub async fn set<K, V>(&self, k: K, v: V) -> Result<(), RedisError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        let mut conn = self.pool.get().await?;
        conn.set(k, v).await.map_err(RedisError::from)
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

pub struct OfferDetailsCache {
    redis: Redis,
}

impl OfferDetailsCache {
    const PREFIX: &str = "maccas:offer_details:";

    pub fn new(redis: Redis) -> Self {
        Self { redis }
    }

    pub async fn set(
        &self,
        details: maccas::caching::OfferDetails,
    ) -> Result<(), OfferDetailsCacheError> {
        let bytes = details.encode_to_vec();
        self.redis
            .set(
                format!("{}:{}", Self::PREFIX, details.proposition_id),
                bytes,
            )
            .await?;

        Ok(())
    }

    pub async fn get_all(
        &self,
        ids: &[i64],
    ) -> Result<Vec<Option<maccas::caching::OfferDetails>>, OfferDetailsCacheError> {
        Ok(self
            .redis
            .mget(
                ids.iter()
                    .map(|id| format!("{}:{}", Self::PREFIX, id))
                    .collect::<Vec<_>>(),
            )
            .await?
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
