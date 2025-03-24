use self::types::{AddOfferInput, AddOfferResponse, RemoveOfferInput};
use crate::{
    graphql::{ValidatedClaims, ValidatedToken},
    settings::Settings,
};
use anyhow::Context as _;
use async_graphql::{Context, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, offers, sea_orm_active_enums::Action};
use event::{CreateEvent, CreateEventResponse, Event};
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    prelude::Uuid, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};
use std::time::Duration;

mod types;

#[derive(Default)]
pub struct OffersMutation;

#[Object]
impl OffersMutation {
    async fn add_offer(
        &self,
        ctx: &Context<'_>,
        input: AddOfferInput,
    ) -> async_graphql::Result<AddOfferResponse> {
        let db = ctx.data::<DatabaseConnection>()?;

        let all_locked_accounts = entity::account_lock::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.id);

        let mut conditions = Condition::all();
        for locked_account in all_locked_accounts {
            conditions = conditions.add(offers::Column::AccountId.ne(locked_account));
        }

        let (offer, account) = offers::Entity::find()
            .filter(conditions)
            .filter(offers::Column::OfferPropositionId.eq(input.offer_proposition_id))
            .find_also_related(accounts::Entity)
            .one(db)
            .await?
            .context("No offer found for this id")?;

        let account = account.context("Must find related account")?;

        let in_15_minutes = chrono::offset::Utc::now().naive_utc() + Duration::from_secs(900);
        entity::account_lock::ActiveModel {
            id: Set(offer.account_id),
            unlock_at: Set(in_15_minutes),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let claims = ctx.data_opt::<ValidatedClaims>();
        let offer_id = offer.id;
        let validated_proposition_id = input.offer_proposition_id;
        let settings = ctx.data::<Settings>()?;

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

        let deal_stack_response = api_client
            .add_to_offers_dealstack(&offer.offer_proposition_id, OFFSET, &input.store_id)
            .await?;

        let deal_stack_response = deal_stack_response
            .body
            .response
            .context("Must have added offer")?;

        let token = ctx.data_opt::<ValidatedToken>().map(|v| &v.0);
        let http_client = ctx.data::<ClientWithMiddleware>()?;

        let transaction_id = Uuid::new_v4();
        entity::offer_audit::ActiveModel {
            action: Set(Action::Add),
            proposition_id: Set(validated_proposition_id),
            user_id: Set(claims.and_then(|c| c.0.user_id.parse().ok())),
            transaction_id: Set(transaction_id),
            ..Default::default()
        }
        .insert(db)
        .await?;

        let cleanup_event = CreateEvent {
            event: Event::Cleanup {
                offer_id,
                transaction_id,
                store_id: input.store_id,
                // account.id
                account_id: offer.account_id,
            },
            delay: Duration::from_secs(900),
        };

        let request_url = format!("{}/{}", settings.event_api_base, CreateEvent::path());
        let request = http_client.post(request_url).json(&cleanup_event);

        let request = if let Some(token) = token {
            request.bearer_auth(token)
        } else {
            request
        };

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let id = response.json::<CreateEventResponse>().await?.id;
                    tracing::info!("created cleanup event with id {}", id);
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(AddOfferResponse {
            id: offer_id,
            code: deal_stack_response.random_code,
        })
    }

    async fn remove_offer(
        &self,
        ctx: &Context<'_>,
        input: RemoveOfferInput,
    ) -> async_graphql::Result<Uuid> {
        let db = ctx.data::<DatabaseConnection>()?;

        let (offer, account) = offers::Entity::find_by_id(input.id)
            .find_also_related(accounts::Entity)
            .one(db)
            .await?
            .context("No offer found for this id")?;

        let account = account.context("Must find related account")?;

        let settings = ctx.data::<Settings>()?;

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

        let response = api_client
            .remove_from_offers_dealstack(
                &offer.offer_id,
                &offer.offer_proposition_id,
                OFFSET,
                &input.store_id,
            )
            .await?;

        let claims = ctx.data_opt::<ValidatedClaims>();

        if response.status.is_success() {
            entity::account_lock::Entity::delete_by_id(offer.account_id)
                .exec(db)
                .await?;

            entity::offer_audit::ActiveModel {
                action: Set(Action::Remove),
                proposition_id: Set(offer.offer_proposition_id),
                // FIXME: find old one ?
                user_id: Set(claims.and_then(|c| c.0.user_id.parse().ok())),
                transaction_id: Set(Uuid::new_v4()),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        Ok(input.id)
    }
}
