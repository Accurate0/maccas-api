use super::Context;
use crate::types::api::Offer;
use crate::{cache, lock};
use async_trait::async_trait;
use http::Response;
use itertools::Itertools;
use lambda_http::{Body, Error, Request};
use simple_dispatcher::Executor;
use std::collections::HashMap;

pub struct Deals;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for Deals {
    async fn execute(&self, _request: &Request, ctx: &Context) -> Result<Response<Body>, Error> {
        let locked_deals =
            lock::get_all_locked_deals(&ctx.dynamodb_client, &ctx.api_config.offer_id_table_name).await?;
        let offer_list = cache::get_all_offers_as_vec(&ctx.dynamodb_client, &ctx.api_config.cache_table_name).await?;

        // filter locked deals & extras
        // 30762 is McCafé®, Buy 5 Get 1 Free, valid till end of year...
        let offer_list: Vec<Offer> = offer_list
            .into_iter()
            .filter(|offer| !locked_deals.contains(&offer.deal_uuid.to_string()))
            .collect();

        let mut count_map = HashMap::<i64, u32>::new();
        for offer in &offer_list {
            match count_map.get(&offer.offer_proposition_id) {
                Some(count) => {
                    let count = count + 1;
                    count_map.insert(offer.offer_proposition_id.clone(), count)
                }
                None => count_map.insert(offer.offer_proposition_id.clone(), 1),
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

        Ok(Response::builder()
            .status(200)
            .body(serde_json::to_string(&offer_list)?.into())?)
    }
}
