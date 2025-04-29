use super::{EventManager, EventManagerError};
use crate::event_manager::handlers::cleanup::cleanup;
use crate::event_manager::handlers::save_image::save_image;
use base::{
    jwt::JwtValidationError,
    retry::{retry_async, ExponentialBackoff, RetryResult},
};
use converters::ConversionError;
use event::Event;
use futures::FutureExt;
use new_offer_found::new_offer_found;
use refresh_points::refresh_points;
use sea_orm::DbErr;
use std::{fmt::Display, num::TryFromIntError, panic::AssertUnwindSafe, time::Duration};
use thiserror::Error;
use tracing::{span, Instrument};

mod cleanup;
mod new_offer_found;
mod refresh_points;
mod save_image;

pub use save_image::S3BucketType;

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
    #[error("A join error occurred: `{0}`")]
    TaskJoinError(#[from] tokio::task::JoinError),
    #[error("An event manager error occurred: `{0}`")]
    EventManagerError(#[from] EventManagerError),
    #[error("A TryFromInt error occurred: `{0}`")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("A ConversionError error occurred: `{0}`")]
    ConversionError(#[from] ConversionError),
    #[error("A JwtValidation error occurred: `{0}`")]
    JwtValidationError(#[from] JwtValidationError),
}

// TODO: make them event manager functions or some kind of trait setup :)
pub async fn handle(event_manager: EventManager) {
    // wait for concurrency limit before processing next item
    // FIXME: is this the best spot?
    let permit = event_manager.acquire_permit().await;

    if let Ok(Some(msg)) = event_manager
        .inner
        .event_queue
        .read(Duration::from_secs(300))
        .await
    {
        let event = msg.message;
        if !event_manager.should_run(event.id).await {
            tracing::info!("skipping event {} as it does not meet criteria", event.id);
            return;
        }

        let event_manager = event_manager.clone();
        // 1st attempt + 5 retries
        let backoff = ExponentialBackoff::new(Duration::from_millis(100), 5);
        let event_name = event.evt.to_string();

        let fut = async move {
            event_manager.set_event_running(event.id).await?;

            let result = AssertUnwindSafe(retry_async(backoff, || async {
                let event_manager = event_manager.clone();
                let evt = event.evt.clone();
                match evt {
                    Event::Cleanup {
                        offer_id,
                        transaction_id,
                        audit_id,
                        store_id,
                        account_id,
                    } => {
                        // FIXME: too many args
                        cleanup(
                            offer_id,
                            audit_id,
                            transaction_id,
                            store_id,
                            account_id,
                            event_manager,
                        )
                        .await
                    }
                    Event::SaveImage { basename, force } => {
                        save_image(basename, force, event_manager).await
                    }
                    Event::RefreshPoints { account_id } => {
                        refresh_points(account_id, event_manager).await
                    }
                    Event::NewOfferFound {
                        offer_proposition_id,
                    } => new_offer_found(offer_proposition_id, event_manager).await,
                }
            }))
            .catch_unwind()
            .await;

            event_manager.archive(msg.msg_id).await?;

            match result {
                Ok(result) => match result {
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
                },
                Err(panic_err) => {
                    let panic_message = {
                        let displayable = panic_err
                            .downcast_ref::<&dyn Display>()
                            .map(|p| p.to_string());
                        let stringable = panic_err
                            .downcast_ref::<&dyn ToString>()
                            .map(|p| p.to_string());

                        displayable
                            .or(stringable)
                            .unwrap_or("no panic message found".to_string())
                    };

                    let err = format!("panic: {:?}", panic_message);
                    tracing::error!("{}", err);
                    event_manager
                        .set_event_completed_in_error(event.id, &err, 99)
                        .await?;
                }
            }

            Ok::<(), HandlerError>(())
        }
        .instrument(span!(
            tracing::Level::INFO,
            "event",
            "otel.name" = format!("event::{}", event_name),
            "message_id" = msg.msg_id
        ));

        tokio::spawn(async move {
            if let Err(e) = fut.await {
                tracing::error!("Error handling event: {}", e);
            }

            drop(permit);
        });
    }
}
