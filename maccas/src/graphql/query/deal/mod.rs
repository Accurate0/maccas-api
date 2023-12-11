use self::types::{Deal, DealCodeInput};
use crate::{
    constants::mc_donalds,
    database::types::OfferDatabase,
    proxy,
    routes::Context,
    types::api::{GetDealsOffer, OfferResponse},
};
use async_graphql::Object;
use itertools::Itertools;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;

mod types;

#[derive(Default)]
pub struct DealsQuery;

#[Object]
impl DealsQuery {
    async fn deal(&self) -> Deal {
        Deal {}
    }
}

#[Object]
impl Deal {
    async fn current_deals<'a>(
        &self,
        gql_ctx: &'a async_graphql::Context<'a>,
    ) -> Result<Vec<GetDealsOffer>, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        let locked_deals = ctx.database.offer_repository.get_locked_offers().await?;
        let offer_list = ctx.database.offer_repository.get_all_offers_vec().await?;

        // filter locked deals
        let mut offer_list: Vec<OfferDatabase> = offer_list
            .into_iter()
            .filter(|offer| !locked_deals.contains(&offer.deal_uuid.to_string()))
            .collect();

        let mut rng = StdRng::from_entropy();
        offer_list.shuffle(&mut rng);

        let mut count_map = HashMap::<i64, i32>::new();
        for offer in &offer_list {
            match count_map.get(&offer.offer_proposition_id) {
                Some(count) => {
                    let count = count + 1;
                    count_map.insert(offer.offer_proposition_id, count)
                }
                None => count_map.insert(offer.offer_proposition_id, 1),
            };
        }

        let offer_list: Vec<GetDealsOffer> = offer_list
            .into_iter()
            .unique_by(|offer| offer.offer_proposition_id)
            .map(|original_offer| {
                let mut offer = GetDealsOffer::from(original_offer.clone());
                offer.count = *count_map.get(&original_offer.offer_proposition_id).unwrap();

                offer
            })
            .collect();

        Ok(offer_list)
    }

    async fn last_refresh<'a>(
        &self,
        gql_ctx: &'a async_graphql::Context<'a>,
    ) -> Result<String, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        ctx.database.refresh_repository.get_last_refresh().await
    }

    async fn code<'a>(
        &self,
        gql_ctx: &'a async_graphql::Context<'a>,
        input: DealCodeInput,
    ) -> Result<OfferResponse, anyhow::Error> {
        // TODO: proxies and retry middleware
        let ctx = gql_ctx.data_unchecked::<Context>();
        let (account, _offer) = ctx.database.offer_repository.get_offer(&input.uuid).await?;
        let proxy = proxy::get_proxy(&ctx.config.proxy).await;
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = ctx
            .database
            .account_repository
            .get_api_client(
                http_client,
                &ctx.config.mcdonalds.client_id,
                &ctx.config.mcdonalds.client_secret,
                &ctx.config.mcdonalds.sensor_data,
                &account,
                false,
            )
            .await?;

        let resp = api_client
            .get_offers_dealstack(mc_donalds::default::OFFSET, &input.store_id)
            .await?;

        Ok(OfferResponse::from(resp.body))
    }
}
