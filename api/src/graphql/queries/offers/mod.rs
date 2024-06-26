use self::types::{Offer, OfferByIdInput, OfferByIdResponse};
use crate::{name_of, settings::Settings};
use anyhow::Context as _;
use async_graphql::{Context, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, offers};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult, JoinType,
    QueryFilter, QuerySelect, RelationTrait,
};
use std::collections::HashMap;

pub mod dataloader;
mod types;

#[derive(FromQueryResult, Debug)]
struct OfferCount {
    offer_proposition_id: i64,
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

        let count_map = if ctx.look_ahead().field("count").exists() {
            Some(
                offers::Entity::find()
                    .select_only()
                    .filter(conditions.clone())
                    .column(offers::Column::OfferPropositionId)
                    .column_as(
                        offers::Column::OfferPropositionId.count(),
                        name_of!(count in OfferCount),
                    )
                    .group_by(offers::Column::OfferPropositionId)
                    .into_model::<OfferCount>()
                    .all(db)
                    .await?
                    .iter()
                    .map(|o| (o.offer_proposition_id, o.count))
                    .collect::<HashMap<_, _>>(),
            )
        } else {
            None
        };

        Ok(offers::Entity::find()
            .distinct_on([offers::Column::OfferPropositionId])
            .filter(conditions)
            .all(db)
            .await?
            .into_iter()
            .map(|o| {
                let count = count_map
                    .as_ref()
                    .and_then(|c| c.get(&o.offer_proposition_id).copied());

                Offer(o, count)
            })
            .collect::<Vec<_>>())
    }
}
