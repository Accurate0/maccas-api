use crate::database::offer::OfferRepository;
use crate::database::types::OfferDatabase;
use crate::routes::Context;
use crate::types::api::GetDealsOffer;
use crate::types::error::ApiError;
use itertools::Itertools;
use rand::prelude::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;

#[utoipa::path(
    responses(
        (status = 200, description = "List of available deals", body = [GetDealsOffer]),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[get("/deals")]
// TODO: optimise, its getting quite slow
// caching in dynamo probs
pub async fn get_deals(
    _ctx: &State<Context>,
    offer_repository: &State<OfferRepository>,
) -> Result<Json<Vec<GetDealsOffer>>, ApiError> {
    let locked_deals = offer_repository.get_all_locked_deals().await?;
    let offer_list = offer_repository.get_all_offers_as_vec().await?;

    // filter locked deals
    let mut offer_list: Vec<OfferDatabase> = offer_list
        .into_iter()
        .filter(|offer| !locked_deals.contains(&offer.deal_uuid.to_string()))
        .collect();

    let mut rng = StdRng::from_entropy();
    offer_list.shuffle(&mut rng);

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

    let offer_list: Vec<GetDealsOffer> = offer_list
        .into_iter()
        .unique_by(|offer| offer.offer_proposition_id)
        .map(|original_offer| {
            let mut offer = GetDealsOffer::from(original_offer.clone());
            offer.count = *count_map.get(&original_offer.offer_proposition_id).unwrap();

            offer
        })
        .collect();

    Ok(Json(offer_list))
}
