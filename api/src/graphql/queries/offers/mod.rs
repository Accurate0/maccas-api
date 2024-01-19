use self::types::{Offer, OfferByIdInput, OfferByIdResponse};
use crate::name_of;
use async_graphql::{Context, Object};
use base::account_manager::AccountManager;
use entity::offers;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, FromQueryResult, QueryFilter,
    QuerySelect,
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
        _ctx: &Context<'a>,
        _input: OfferByIdInput,
    ) -> async_graphql::Result<OfferByIdResponse> {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(OfferByIdResponse { code: "rm".into() })
    }

    async fn offers<'a>(&self, ctx: &Context<'a>) -> async_graphql::Result<Vec<Offer>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let account_manager = ctx.data::<AccountManager>()?;
        let all_locked_accounts = account_manager.get_all_locked().await?;

        tracing::info!("locked accounts: {:?}", all_locked_accounts);
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
