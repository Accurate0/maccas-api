use crate::{
    database::offer::OfferRepository,
    guards::admin::AdminOnlyRoute,
    types::{api::AdminLockedDealsResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of currently locked deals", body = AdminLockedDealsResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[get("/admin/locked-deals")]
pub async fn get_locked_deals(
    offer_repo: &State<OfferRepository>,
    _admin: AdminOnlyRoute,
) -> Result<Json<AdminLockedDealsResponse>, ApiError> {
    let locked_deals = offer_repo.get_locked_offers().await?;
    Ok(Json(AdminLockedDealsResponse(locked_deals)))
}
