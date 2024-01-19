use super::EventManager;
use crate::event_manager::handlers::cleanup::cleanup;
use base::{
    account_manager::AccountManagerError,
    retry::{retry_async, ExponentialBackoff, RetryResult},
};
use event::Event;
use sea_orm::DbErr;
use state::TypeMap;
use std::{sync::Arc, time::Duration};
use thiserror::Error;

mod cleanup;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Serializer error has ocurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has ocurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
    #[error("An unknown error ocurred: `{0}`")]
    UnknownError(#[from] anyhow::Error),
    #[error("An reqwest ocurred: `{0}`")]
    ReqwestError(#[from] reqwest::Error),
    #[error("McDonald's client error occurred: `{0}`")]
    McDonaldsClientError(#[from] libmaccas::ClientError),
    #[error("Account manager error occurred: `{0}`")]
    AccountManagerError(#[from] AccountManagerError),
}

// TODO: make them event manager functions or some kind of trait setup :)
pub async fn handle(event_manager: EventManager, type_map: Arc<TypeMap![Sync + Send]>) {
    if let Some(event) = event_manager.inner.event_queue.pop().await {
        let event_manager = event_manager.clone();
        // 1st attempt + 5 retries
        let backoff = ExponentialBackoff::new(Duration::from_millis(100), 5);

        tokio::spawn(async move {
            let result = match event.evt {
                Event::Cleanup {
                    offer_id,
                    transaction_id,
                    store_id,
                } => {
                    retry_async(backoff, || {
                        cleanup(
                            offer_id,
                            transaction_id,
                            store_id.clone(),
                            event_manager.clone(),
                            type_map.clone(),
                        )
                    })
                    .await
                }
            };

            match result {
                RetryResult::Ok { attempts, .. } => {
                    tracing::info!("success: with {} attempts", attempts);
                    match event_manager
                        .set_retry_attempts(event.id, attempts.try_into().unwrap())
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => tracing::error!("error incrementing attempt: {}", e),
                    };
                }
                RetryResult::Err { attempts, value } => {
                    tracing::error!("error: {} with {} attempts", value, attempts);
                    match event_manager
                        .set_retry_attempts(event.id, attempts.try_into().unwrap())
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => tracing::error!("error incrementing attempt: {}", e),
                    };

                    match event_manager
                        .set_event_error(event.id, &value.to_string())
                        .await
                    {
                        Ok(_) => {}
                        Err(e) => tracing::error!("error setting error: {}", e),
                    };
                }
            }

            if let Err(e) = event_manager.complete_event(event.id).await {
                tracing::error!("error marking event as completed: {}", e)
            };
        });
    }
}
