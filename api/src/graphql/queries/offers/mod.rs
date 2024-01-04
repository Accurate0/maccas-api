use self::types::{Offer, OfferByIdInput, OfferByIdResponse};
use crate::name_of;
use async_graphql::{Context, Object};
use entity::offers;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, QuerySelect};
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

        let fetch_count = ctx.look_ahead().field("count").exists();

        let count_map = if fetch_count {
            Some(
                offers::Entity::find()
                    .select_only()
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
