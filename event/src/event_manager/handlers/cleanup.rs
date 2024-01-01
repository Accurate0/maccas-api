use super::HandlerError;
use event::CreateEvent;
use sea_orm::DatabaseConnection;

pub async fn cleanup(offer_id: String, _db: DatabaseConnection) -> Result<(), HandlerError> {
    tracing::info!("cleanup for {}", offer_id);
    serde_json::from_str::<CreateEvent>("x")?;
    Ok(())
}
