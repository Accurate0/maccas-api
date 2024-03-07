use super::EventManager;
use crate::event_manager::handlers::cleanup::cleanup;
use base::retry::{retry_async, ExponentialBackoff, RetryResult};
use event::Event;
use sea_orm::DbErr;
use std::time::Duration;
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
}

// TODO: make them event manager functions or some kind of trait setup :)
pub async fn handle(event_manager: EventManager) {
    if let Some(event) = event_manager.inner.event_queue.pop().await {
        let event_manager = event_manager.clone();
        // 1st attempt + 5 retries
        let backoff = ExponentialBackoff::new(Duration::from_millis(100), 5);

        let fut = async move {
            event_manager.set_event_running(event.id).await?;

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
                        )
                    })
                    .await
                }
            };

            match result {
                RetryResult::Ok { attempts, .. } => {
                    tracing::info!("success: with {} attempts", attempts);

                    event_manager
                        .set_event_completed(event.id, attempts.try_into()?)
                        .await?;
                }
                RetryResult::Err { attempts, value } => {
                    tracing::error!("error: {} with {} attempts", value, attempts);

                    event_manager
                        .set_event_completed_in_error(
                            event.id,
                            &value.to_string(),
                            attempts.try_into()?,
                        )
                        .await?;
                }
            }

            Ok::<(), anyhow::Error>(())
        };

        tokio::spawn(async move {
            if let Err(e) = fut.await {
                tracing::error!("Error handling event: {}", e);
            }
        });
    }
}
