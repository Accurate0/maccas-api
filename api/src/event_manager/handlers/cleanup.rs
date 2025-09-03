use super::HandlerError;
use crate::{event_manager::EventManager, settings::Settings};
use anyhow::Context;
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, concurrent_active_deals, offers, sea_orm_active_enums::Action};
use opentelemetry::trace::TraceContextExt;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Unchanged, ColumnTrait, EntityTrait, QueryFilter, Set,
    TransactionTrait, prelude::Expr, sea_query::OnConflict,
};
use std::time::Duration;
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(em))]
pub async fn cleanup(
    offer_id: Uuid,
    // you can get transaction id from audit id
    audit_id: i32,
    transaction_id: Uuid,
    store_id: String,
    account_id: Uuid,
    user_id: Option<Uuid>,
    em: EventManager,
) -> Result<(), HandlerError> {
    tracing::info!("cleanup for {}", offer_id);

    let settings = em.get_state::<Settings>();
    let db = em.db();

    let fut = async {
        let (offer, account) = offers::Entity::find_by_id(offer_id)
            .find_also_related(accounts::Entity)
            .one(db)
            .await?
            .context("No offer found for this id")?;

        let account = account.context("Must find matching account")?;

        let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
            .basic_auth(&settings.proxy.username, &settings.proxy.password);

        let account_lock_txn = db.begin().await?;
        let api_client = base::maccas::get_activated_maccas_api_client(
            account,
            proxy,
            &settings.mcdonalds.client_id,
            &account_lock_txn,
        )
        .await?;
        account_lock_txn.commit().await?;

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
            tracing::info!("not found in deal stack, marking {transaction_id} as likely used");

            let likely_used_update = entity::offer_audit::ActiveModel {
                id: Unchanged(audit_id),
                likely_used: Set(Some(true)),
                transaction_id: Set(transaction_id),
                ..Default::default()
            };

            entity::offer_audit::Entity::update(likely_used_update)
                .filter(entity::offer_audit::Column::Id.eq(audit_id))
                .exec(db)
                .await?;
        }

        if let Some(user_id) = user_id {
            let active_deals_model = concurrent_active_deals::ActiveModel {
                user_id: Set(user_id),
                count: Set(0),
            };

            concurrent_active_deals::Entity::insert(active_deals_model)
                .on_conflict(
                    OnConflict::column(concurrent_active_deals::Column::UserId)
                        .value(
                            concurrent_active_deals::Column::Count,
                            Expr::cust_with_expr(
                                "GREATEST($1, 0)",
                                Expr::column((
                                    concurrent_active_deals::Entity,
                                    concurrent_active_deals::Column::Count,
                                ))
                                .sub(Expr::value(1)),
                            ),
                        )
                        .to_owned(),
                )
                .exec(db)
                .await?;
        }

        Ok::<(), HandlerError>(())
    };

    let result = fut.await;

    entity::account_lock::Entity::delete_by_id(account_id)
        .exec(db)
        .await?;

    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id()
        .to_string();

    let _ = em
        .create_event(
            api::Event::RefreshAccount { account_id },
            Duration::from_secs(10),
            trace_id,
        )
        .await;

    result
}
