use crate::{
    guards::admin::AdminOnlyRoute,
    routes,
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
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
) -> Result<Json<AdminLockedDealsResponse>, ApiError> {
    let locked_deals = ctx.database.get_all_locked_deals().await?;
    Ok(Json(AdminLockedDealsResponse(locked_deals)))
}
