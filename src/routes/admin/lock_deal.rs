use crate::{
    constants::DEFAULT_LOCK_TTL_HOURS, guards::admin::AdminOnlyRoute, routes,
    types::error::ApiError,
};
use chrono::Duration;
use rocket::{http::Status, State};

#[utoipa::path(
    responses(
        (status = 204, description = "Lock this deal"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, Header, description = "Key to bypass JWT checks"),
    ),
)]
#[post("/admin/locked-deals/<deal_id>?<duration>")]
pub async fn lock_deal(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
    deal_id: &str,
    duration: Option<i64>,
) -> Result<Status, ApiError> {
    ctx.database
        .lock_deal(
            deal_id,
            duration.map_or(Duration::hours(DEFAULT_LOCK_TTL_HOURS), |s| {
                Duration::seconds(s)
            }),
        )
        .await?;

    Ok(Status::NoContent)
}
