use super::HandlerError;
use crate::{event_manager::EventManager, jobs::shared::offer_details_model_to_cache};
use caching::OfferDetailsCache;
use sea_orm::EntityTrait;
use tracing::instrument;

#[instrument(skip(em))]
pub async fn populate_offer_details_cache(em: EventManager) -> Result<(), HandlerError> {
    if let Some(caching) = em.try_get_state::<OfferDetailsCache>() {
        let all_offer_details = entity::offer_details::Entity::find().all(em.db()).await?;
        tracing::info!("total: {}", all_offer_details.len());
        for details in all_offer_details {
            caching.set(offer_details_model_to_cache(&details)).await?;
        }
    }

    Ok(())
}

#[instrument(skip(em))]
pub async fn populate_offer_details_cache_for(
    proposition_id: i64,
    em: EventManager,
) -> Result<(), HandlerError> {
    if let Some(caching) = em.try_get_state::<OfferDetailsCache>() {
        let all_offer_details = entity::offer_details::Entity::find_by_id(proposition_id)
            .one(em.db())
            .await?;
        if let Some(details) = all_offer_details {
            caching.set(offer_details_model_to_cache(&details)).await?;
        }
    }

    Ok(())
}
