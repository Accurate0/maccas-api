use futures::StreamExt;
use redis::{AsyncCommands, RedisError};
use sea_orm::prelude::Uuid;
use std::{future::IntoFuture, str::FromStr, time::Duration};
use thiserror::Error;
use tokio::sync::Mutex;

pub struct AccountManager {
    db: Mutex<redis::aio::Connection>,
}

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error("A redis error ocurred: `{0}`")]
    RedisError(#[from] RedisError),
}

impl AccountManager {
    const PREFIX: &'static str = "account-manager";

    pub async fn new(connection_string: &str) -> Result<Self, AccountManagerError> {
        Ok(Self {
            db: redis::Client::open(connection_string)?
                .get_async_connection()
                .await?
                .into(),
        })
    }

    fn get_key_format(id: Uuid) -> String {
        format!("{}-{id}", Self::PREFIX)
    }

    pub async fn lock(
        &self,
        account_id: Uuid,
        expiry: Duration,
    ) -> Result<(), AccountManagerError> {
        self.db
            .lock()
            .await
            .set_ex(
                Self::get_key_format(account_id),
                account_id.to_string(),
                expiry.as_secs(),
            )
            .await?;
        Ok(())
    }

    // FIXME: GROSS
    pub async fn get_all_locked(&self) -> Result<Vec<Uuid>, AccountManagerError> {
        Ok(self
            .db
            .lock()
            .await
            .scan_match::<&str, String>(&format!("{}-*", Self::PREFIX))
            .await?
            .map(|s| Uuid::from_str(&s.replace(format!("{}-", Self::PREFIX).as_str(), "")).unwrap())
            .collect::<Vec<_>>()
            .into_future()
            .await)
    }

    pub async fn unlock(&self, account_id: Uuid) -> Result<bool, AccountManagerError> {
        Ok(self
            .db
            .lock()
            .await
            .del(Self::get_key_format(account_id))
            .await?)
    }
}
