use super::EventManager;
use crate::event_manager::handlers::cleanup::cleanup;
use event::Event;
use sea_orm::DbErr;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

mod cleanup;

#[derive(Error, Debug)]
pub enum HandlerError {
    #[error("Serializer error has ocurred: `{0}`")]
    Serializer(#[from] serde_json::Error),
    #[error("Database error has ocurred: `{0}`")]
    Database(#[from] DbErr),
    #[error("Chrono out of range error has ocurred: `{0}`")]
    OutOfRangeError(#[from] chrono::OutOfRangeError),
}

pub async fn handle(event_manager: EventManager, cancellation_token: CancellationToken) {
    // TODO: cancellation token
    // TODO: persistence

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("handle cancelled");
                break;
            },
            Some(event) = event_manager.inner.event_queue.pop() => {
                let db = event_manager.inner.db.clone();
                let event_manager = event_manager.clone();

                tokio::spawn(async move {
                    match event_manager.increment_event_attempt(event.id).await {
                        Ok(_) => {},
                        Err(e) => tracing::error!("error incrementing attempt: {}", e),
                    };

                    // RETRY IF ANY OF THIS FAILS
                    if let Err(e) = match event.evt {
                        Event::Cleanup { offer_id } =>  {
                            cleanup(offer_id, db).await
                        }
                    } {
                        if let Err(e) = event_manager.set_event_error(event.id, &e.to_string()).await {
                            tracing::error!("error marking event as error: {}", e)
                        }
                    };


                    if let Err(e) = event_manager.complete_event(event.id).await {
                        tracing::error!("error marking event as completed: {}", e)
                    };
                });
            }
        }
    }
}
