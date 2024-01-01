use crate::events::handlers::cleanup::cleanup;
use crate::events::Event;
use base::delay_queue::DelayQueue;
use sea_orm::DatabaseConnection;
use tokio_util::sync::CancellationToken;

mod cleanup;

pub async fn handle(
    task_queue: DelayQueue<Event>,
    db: DatabaseConnection,
    cancellation_token: CancellationToken,
) {
    // TODO: cancellation token
    // TODO: persistence

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("handle cancelled");
                break;
            },
            Some(task) = task_queue.pop() => {
                match task {
                    Event::Cleanup { offer_id } => tokio::spawn(cleanup(offer_id, db.clone())),
                };
            }
        }
    }
}
