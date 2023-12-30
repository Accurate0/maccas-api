use sea_orm::DatabaseConnection;

pub(crate) async fn cleanup(offer_id: String, _db: DatabaseConnection) {
    tracing::info!("cleanup for {}", offer_id);
}
