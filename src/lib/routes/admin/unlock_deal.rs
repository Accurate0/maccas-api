use crate::{guards::admin::AdminOnlyRoute, routes, types::error::ApiError};
use rocket::{http::Status, State};

#[utoipa::path(
    responses(
        (status = 204, description = "Unlocked deal"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
    params(
        ("Authorization" = String, header, description = "Valid JWT with user id in allowed list"),
    ),
)]
#[delete("/admin/locked-deals/<deal_id>")]
pub async fn unlock_deal(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
    deal_id: &str,
) -> Result<Status, ApiError> {
    ctx.database.unlock_deal(deal_id).await?;

    Ok(Status::NoContent)
}
