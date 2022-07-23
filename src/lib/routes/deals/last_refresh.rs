use crate::routes::Context;
use crate::types::api::LastRefreshInformation;
use crate::types::error::ApiError;
use rocket::serde::json::Json;
use rocket::State;

pub struct LastRefresh;

#[utoipa::path(
        get,
        path = "/deals/last-refresh",
        responses(
            (status = 200, description = "Last Refresh of Cache", body = LastRefreshInformation),
            (status = 500, description = "Internal Server Error"),
        ),
        tag = "deals",
    )]
#[get("/deals/last-refresh")]
pub async fn last_refresh(ctx: &State<Context<'_>>) -> Result<Json<LastRefreshInformation>, ApiError> {
    let response = ctx.database.get_refresh_time_for_offer_cache().await?;
    Ok(Json(LastRefreshInformation { last_refresh: response }))
}
