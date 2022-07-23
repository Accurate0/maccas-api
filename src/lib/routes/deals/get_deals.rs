use crate::routes::Context;
use crate::types::api::Offer;
use crate::types::error::ApiError;
use itertools::Itertools;
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;

#[utoipa::path(
    get,
    path = "/deals",
    responses(
        (status = 200, description = "List of available deals", body = [Offer]),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[get("/deals")]
pub async fn get_deals(ctx: &State<Context<'_>>) -> Result<Json<Vec<Offer>>, ApiError> {
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

    Ok(Json(offer_list))
}
