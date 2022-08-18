use crate::{
    routes,
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
    ctx: &State<routes::Context<'_>>,
) -> Result<Json<AccountResponse>, ApiError> {
    let offers = ctx.database.get_all_offers_as_map().await?;
    Ok(Json(AccountResponse::from(offers)))
}
