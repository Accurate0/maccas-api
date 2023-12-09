use crate::{
    database::offer::OfferRepository,
    types::{api::TotalAccountsResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Total account count", body = i64),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "statistics",
)]
#[get("/statistics/total-accounts")]
pub async fn get_total_accounts(
    offer_repository: &State<OfferRepository>,
) -> Result<Json<TotalAccountsResponse>, ApiError> {
    let offers = offer_repository.get_all_offers().await?;
    Ok(Json(TotalAccountsResponse(offers.len() as i64)))
}
