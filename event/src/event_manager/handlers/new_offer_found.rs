use super::HandlerError;
use crate::event_manager::EventManager;
use tracing::instrument;

#[instrument(skip(_em))]
pub async fn new_offer_found(
    _offer_proposition_id: i64,
    _em: EventManager,
) -> Result<(), HandlerError> {
    Ok(())
}
