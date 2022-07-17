use crate::routes::Context;
use crate::types::api::Offer;
use async_trait::async_trait;
use http::Response;
use itertools::Itertools;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};
use std::collections::HashMap;

pub struct Deals;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/deals",
        responses(
            (status = 200, description = "List of available deals", body = [Offer]),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "deals",
    )]
    pub fn deals() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Deals {
    async fn execute(&self, ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let locked_deals = ctx.database.get_all_locked_deals().await?;
        let offer_list = ctx.database.get_all_offers_as_vec().await?;

        // filter locked deals
        let offer_list: Vec<Offer> = offer_list
            .into_iter()
            .filter(|offer| !locked_deals.contains(&offer.deal_uuid.to_string()))
            .collect();

        let mut count_map = HashMap::<i64, u32>::new();
        for offer in &offer_list {
            match count_map.get(&offer.offer_proposition_id) {
                Some(count) => {
                    let count = count + 1;
                    count_map.insert(offer.offer_proposition_id, count)
                }
                None => count_map.insert(offer.offer_proposition_id, 1),
            };
        }

        let offer_list: Vec<Offer> = offer_list
            .into_iter()
            .unique_by(|offer| offer.offer_proposition_id)
            .map(|mut offer| {
                offer.count = *count_map.get(&offer.offer_proposition_id).unwrap();
                offer
            })
            .collect();

        Ok(serde_json::to_value(&offer_list)?.into_response())
    }
}
