use crate::{database::types::OfferDatabase, routes, types::api::GetDealsOffer};
use itertools::Itertools;
use juniper::graphql_object;
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[graphql_object(context = crate::routes::Context)]
/// The root query object of the schema
impl Query {
    async fn deals(
        #[graphql(context)] ctx: &routes::Context,
    ) -> Result<Vec<GetDealsOffer>, anyhow::Error> {
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
}
