use self::types::{Offer, OfferByIdInput, OfferByIdResponse};
use crate::{graphql::ValidatedClaims, settings::Settings};
use anyhow::Context as _;
use async_graphql::{Context, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, offer_details, offers, recommendations};
use sea_orm::{
    prelude::Uuid, ColumnTrait, Condition, DatabaseConnection, EntityTrait, JoinType, Order,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait, TransactionTrait,
};
use std::str::FromStr;

pub mod dataloader;
mod types;

#[derive(Default)]
pub struct OffersQuery;

#[Object]
impl OffersQuery {
    async fn offer_by_id(
        &self,
        ctx: &Context<'_>,
        input: OfferByIdInput,
    ) -> async_graphql::Result<OfferByIdResponse> {
        let db = ctx.data::<DatabaseConnection>()?;
        let settings = ctx.data::<Settings>()?;

        let models = offers::Entity::find_by_id(input.id)
            .select_also(accounts::Entity)
            .join(JoinType::LeftJoin, offers::Relation::Accounts.def())
            .one(db)
            .await?
            .context("must find offer by id")?;

        let account = models.1.context("must have matching account")?;

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

        let offer_code = api_client
            .get_offers_dealstack(OFFSET, &input.store_id)
            .await?
            .body
            .response
            .context("must have deal stack response")?;

        Ok(OfferByIdResponse {
            code: offer_code.random_code,
        })
    }

    async fn upcoming_offers(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Offer>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = Condition::all().add(offers::Column::ValidFrom.gt(now));

        Ok(offers::Entity::find()
            .find_also_related(offer_details::Entity)
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|(offer, _offer_details)| Offer(offer))
            .collect())
    }

    async fn recommendations(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Offer>> {
        let claims = ctx.data_opt::<ValidatedClaims>().map(|c| c.0.clone());
        if claims.is_none() {
            return Ok(vec![]);
        }

        let claims = claims.unwrap();
        let db = ctx.data::<DatabaseConnection>()?;
        let recommendations = entity::recommendations::Entity::find()
            .filter(recommendations::Column::UserId.eq(Uuid::from_str(&claims.user_id)?))
            .one(db)
            .await?
            .map(|m| m.offer_proposition_ids)
            .unwrap_or_default();

        let all_locked_accounts = entity::account_lock::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.id);

        let mut conditions = Condition::all();
        for locked_account in all_locked_accounts {
            conditions = conditions.add(offers::Column::AccountId.ne(locked_account));
        }

        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = conditions
            .add(offers::Column::ValidTo.gt(now))
            .add(offers::Column::ValidFrom.lt(now))
            .add(offers::Column::OfferPropositionId.is_in(recommendations));

        let offers = offers::Entity::find()
            .distinct_on([offer_details::Column::ShortName])
            .find_also_related(offer_details::Entity)
            .order_by(offer_details::Column::ShortName, Order::Asc)
            .order_by(offers::Column::ValidTo, Order::Asc)
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|(model, _)| Offer(model))
            .collect::<Vec<_>>();

        Ok(offers)
    }

    async fn offers(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<Offer>> {
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

        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = conditions
            .add(offers::Column::ValidTo.gt(now))
            .add(offers::Column::ValidFrom.lt(now));

        Ok(offers::Entity::find()
            .distinct_on([offer_details::Column::ShortName])
            .find_also_related(offer_details::Entity)
            .order_by(offer_details::Column::ShortName, Order::Asc)
            .order_by(offers::Column::ValidTo, Order::Asc)
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|(offer, _offer_details)| Offer(offer))
            .collect::<Vec<_>>())
    }
}
