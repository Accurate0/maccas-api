use self::types::{Offer, OfferByIdInput, OfferByIdResponse};
use crate::{name_of, settings::Settings};
use anyhow::Context as _;
use async_graphql::{Context, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, offer_details, offers};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult, JoinType, Order,
    QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use std::collections::HashMap;

pub mod dataloader;
mod types;

#[derive(FromQueryResult, Debug)]
struct OfferCount {
    short_name: String,
    count: i64,
}

#[derive(Default)]
pub struct OffersQuery;

#[Object]
impl OffersQuery {
    async fn offer_by_id<'a>(
        &self,
        ctx: &Context<'a>,
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

        let api_client = base::maccas::get_activated_maccas_api_client(
            account,
            proxy,
            &settings.mcdonalds.client_id,
            db,
        )
        .await?;

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

    async fn upcoming_offers<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<Vec<Offer>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = Condition::all().add(offers::Column::ValidFrom.gt(now));

        let count_map = if ctx.look_ahead().field("count").exists() {
            Some(
                offers::Entity::find()
                    .select_only()
                    .join(JoinType::InnerJoin, offers::Relation::OfferDetails.def())
                    .column(offer_details::Column::ShortName)
                    .column_as(
                        offer_details::Column::ShortName.count(),
                        name_of!(count in OfferCount),
                    )
                    .filter(conditions.clone())
                    .group_by(offer_details::Column::ShortName)
                    .into_model::<OfferCount>()
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|o| (o.short_name, o.count))
                    .collect::<HashMap<_, _>>(),
            )
        } else {
            None
        };

        Ok(offers::Entity::find()
            .find_also_related(offer_details::Entity)
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|(offer, offer_details)| {
                let count = if let Some(offer_details) = offer_details {
                    count_map
                        .as_ref()
                        .and_then(|c| c.get(&offer_details.short_name).copied())
                        .unwrap_or(0)
                } else {
                    0
                };

                Offer(offer, Some(count))
            })
            .collect())
    }

    async fn offers<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<Vec<Offer>> {
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

        let count_map = if ctx.look_ahead().field("count").exists() {
            Some(
                offers::Entity::find()
                    .select_only()
                    .filter(conditions.clone())
                    .join(JoinType::InnerJoin, offers::Relation::OfferDetails.def())
                    .column(offer_details::Column::ShortName)
                    .column_as(
                        offer_details::Column::ShortName.count(),
                        name_of!(count in OfferCount),
                    )
                    .group_by(offer_details::Column::ShortName)
                    .into_model::<OfferCount>()
                    .all(db)
                    .await?
                    .into_iter()
                    .map(|o| (o.short_name, o.count))
                    .collect::<HashMap<_, _>>(),
            )
        } else {
            None
        };

        Ok(offers::Entity::find()
            .distinct_on([offer_details::Column::ShortName])
            .find_also_related(offer_details::Entity)
            .order_by(offer_details::Column::ShortName, Order::Asc)
            .order_by(offers::Column::ValidTo, Order::Asc)
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|(offer, offer_details)| {
                let count = if let Some(offer_details) = offer_details {
                    count_map
                        .as_ref()
                        .and_then(|c| c.get(&offer_details.short_name).copied())
                        .unwrap_or(0)
                } else {
                    0
                };

                Offer(offer, Some(count))
            })
            .collect::<Vec<_>>())
    }
}
