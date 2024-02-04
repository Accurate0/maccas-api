use super::HandlerError;
use crate::{event_manager::EventManager, settings::Settings};
use anyhow::Context as _;
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, offers, sea_orm_active_enums::Action};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;

pub async fn cleanup(
    offer_id: Uuid,
    transaction_id: Uuid,
    store_id: String,
    em: EventManager,
) -> Result<(), HandlerError> {
    tracing::info!("cleanup for {}", offer_id);

    let settings = em.get_state::<Settings>();
    let db = em.db();
    let offer = offers::Entity::find_by_id(offer_id)
        .one(db)
        .await?
        .context("No offer found for this id")?;

    let account = accounts::Entity::find_by_id(offer.account_id)
        .one(db)
        .await?
        .context("Must find related account")?;

    let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
        .basic_auth(&settings.proxy.username, &settings.proxy.password);

    let api_client = base::maccas::get_activated_maccas_api_client(
        account,
        proxy,
        &settings.mcdonalds.client_id,
        db,
    )
    .await?;

    let is_in_deal_stack = api_client
        .get_offers_dealstack(OFFSET, &store_id)
        .await?
        .body
        .response
        .and_then(|r| {
            r.deal_stack.map(|s| {
                s.iter().any(|o| {
                    o.offer_id == offer.offer_id
                        && o.offer_proposition_id == offer.offer_proposition_id.to_string()
                })
            })
        })
        .unwrap_or(false);

    if is_in_deal_stack {
        let response = api_client
            .remove_from_offers_dealstack(
                &offer.offer_id,
                &offer.offer_proposition_id,
                OFFSET,
                &store_id,
            )
            .await;

        match response {
            Ok(r) => {
                tracing::info!("deal stack response: {r:?}");
                entity::offer_audit::ActiveModel {
                    action: Set(Action::Remove),
                    proposition_id: Set(offer.offer_proposition_id),
                    transaction_id: Set(transaction_id),
                    ..Default::default()
                }
                .insert(db)
                .await?;
            }
            Err(e) => tracing::error!("error checking dealstack: {}", e),
        }
    } else {
        tracing::info!("not found in deal stack, skipped");
    }

    entity::account_lock::Entity::delete_by_id(offer.account_id)
        .exec(db)
        .await?;

    Ok(())
}
