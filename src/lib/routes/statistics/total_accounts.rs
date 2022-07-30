use crate::{
    routes,
    types::{api::TotalAccountsResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    get,
    path = "/statistics/total-accounts",
    responses(
        (status = 200, description = "Total account count", body = i64),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "statistics",
)]
#[get("/statistics/total-accounts")]
pub async fn get_total_accounts(
    ctx: &State<routes::Context<'_>>,
) -> Result<Json<TotalAccountsResponse>, ApiError> {
    let offers = ctx.database.get_all_offers_as_map().await?;
    Ok(Json(TotalAccountsResponse(offers.len() as i64)))
}
