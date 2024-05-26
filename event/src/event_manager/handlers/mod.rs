use self::save_image::save_image;

use super::EventManager;
use crate::event_manager::handlers::cleanup::cleanup;
use base::retry::{retry_async, ExponentialBackoff, RetryResult};
use event::Event;
use sea_orm::DbErr;
use std::time::Duration;
use thiserror::Error;
use tracing::{span, Instrument};

mod cleanup;
mod save_image;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Serializer error has occurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has occurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has occurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
    #[error("An unknown error occurred: `{0}`")]
    UnknownError(#[from] anyhow::Error),
    #[error("An reqwest occurred: `{0}`")]
    ReqwestError(#[from] reqwest::Error),
    #[error("McDonald's client error occurred: `{0}`")]
    McDonaldsClientError(#[from] libmaccas::ClientError),
    #[error("S3 Credentials error occurred: `{0}`")]
    S3CredentialsError(#[from] s3::creds::error::CredentialsError),
    #[error("Http Creation error occurred: `{0}`")]
    HttpCreationError(#[from] base::http::HttpCreationError),
    #[error("A reqwest middleware error occurred: `{0}`")]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error("A s3 error occurred: `{0}`")]
    S3Error(#[from] s3::error::S3Error),
    #[error("An image error occurred: `{0}`")]
    ImageError(#[from] image::ImageError),
    #[error("An io error occurred: `{0}`")]
    IOError(#[from] std::io::Error),
}

// TODO: make them event manager functions or some kind of trait setup :)
pub async fn handle(event_manager: EventManager) {
    if let Some(event) = event_manager.inner.event_queue.pop().await {
        // wait for concurrency limit before processing next item
        // FIXME: is this the best spot?
        let permit = event_manager.acquire_permit().await;
        let event_manager = event_manager.clone();
        // 1st attempt + 5 retries
        let backoff = ExponentialBackoff::new(Duration::from_millis(100), 5);
        let event_name = event.evt.to_string();

        let fut = async move {
            event_manager.set_event_running(event.id).await?;

            let result = retry_async(backoff, || async {
                let event_manager = event_manager.clone();
                let evt = event.evt.clone();
                match evt {
                    Event::Cleanup {
                        offer_id,
                        transaction_id,
                        store_id,
                    } => cleanup(offer_id, transaction_id, store_id, event_manager).await,
                    Event::SaveImage { basename } => save_image(basename, event_manager).await,
                }
            })
            .await;

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
        }
        .instrument(span!(
            tracing::Level::INFO,
            "event",
            "otel.name" = format!("event::{}", event_name)
        ));

        tokio::spawn(async move {
            if let Err(e) = fut.await {
                tracing::error!("Error handling event: {}", e);
            }

            drop(permit);
        });
    }
}
