use crate::{
    database::offer::OfferRepository,
    types::{api::AccountResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Account statistics", body = AccountResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "statistics",
)]
#[get("/statistics/account")]
pub async fn get_accounts(
    offer_repository: &State<OfferRepository>,
) -> Result<Json<AccountResponse>, ApiError> {
    let offers = offer_repository.get_all_offers().await?;
    Ok(Json(AccountResponse::from(offers)))
}
