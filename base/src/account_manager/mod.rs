use crate::retry::{retry_async, ExponentialBackoff};
use futures::StreamExt;
use redis::{AsyncCommands, RedisError};
use sea_orm::prelude::Uuid;
use std::{future::IntoFuture, str::FromStr, time::Duration};
use thiserror::Error;
use tokio::sync::Mutex;

pub struct AccountManager {
    db: Mutex<redis::aio::ConnectionManager>,
}

#[derive(Error, Debug)]
pub enum AccountManagerError {
    #[error("A redis error ocurred: `{0}`")]
    RedisError(#[from] RedisError),
}

impl AccountManager {
    const PREFIX: &'static str = "account-manager";
    const ATTEMPTS: u64 = 5;
    const DELAY: Duration = Duration::from_millis(100);

    pub async fn new(connection_string: &str) -> Result<Self, AccountManagerError> {
        Ok(Self {
            db: redis::Client::open(connection_string)?
                .get_connection_manager()
                .await?
                .into(),
        })
    }

    fn get_key_format(id: Uuid) -> String {
        format!("{}-{id}", Self::PREFIX)
    }

    pub async fn ping(&self) -> bool {
        self.db
            .lock()
            .await
            .send_packed_command(&redis::cmd("PING"))
            .await
            .is_ok()
    }

    pub async fn lock(
        &self,
        account_id: Uuid,
        expiry: Duration,
    ) -> Result<(), AccountManagerError> {
        let backoff = ExponentialBackoff::new(Self::DELAY, Self::ATTEMPTS);
        retry_async(backoff, || async move {
            self.db
                .lock()
                .await
                .set_ex::<_, _, String>(
                    Self::get_key_format(account_id),
                    account_id.to_string(),
                    expiry.as_secs(),
                )
                .await
        })
        .await;

        Ok(())
    }

    // FIXME: GROSS
    pub async fn get_all_locked(&self) -> Result<Vec<Uuid>, AccountManagerError> {
        let backoff = ExponentialBackoff::new(Self::DELAY, Self::ATTEMPTS);
        retry_async(backoff, || async move {
            Ok(self
                .db
                .lock()
                .await
                .scan_match::<&str, String>(&format!("{}-*", Self::PREFIX))
                .await?
                .map(|s| {
                    Uuid::from_str(&s.replace(format!("{}-", Self::PREFIX).as_str(), "")).unwrap()
                })
                .collect::<Vec<_>>()
                .into_future()
                .await)
        })
        .await
        .into()
    }

    pub async fn unlock(&self, account_id: Uuid) -> Result<bool, AccountManagerError> {
        let backoff = ExponentialBackoff::new(Self::DELAY, Self::ATTEMPTS);
        retry_async(backoff, || async move {
            Ok(self
                .db
                .lock()
                .await
                .del(Self::get_key_format(account_id))
                .await?)
        })
        .await
        .into()
    }
}
