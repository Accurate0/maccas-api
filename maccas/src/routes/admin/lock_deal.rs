use crate::{
    constants::config::DEFAULT_LOCK_TTL_HOURS, database::offer::OfferRepository,
    guards::admin::AdminOnlyRoute, types::error::ApiError,
};
use chrono::Duration;
use rocket::{http::Status, State};

#[utoipa::path(
    responses(
        (status = 204, description = "Lock this deal"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[post("/admin/locked-deals/<deal_id>?<duration>")]
pub async fn lock_deal(
    offer_repo: &State<OfferRepository>,
    _admin: AdminOnlyRoute,
    deal_id: &str,
    duration: Option<i64>,
) -> Result<Status, ApiError> {
    offer_repo
        .lock_deal(
            deal_id,
            duration.map_or(Duration::hours(DEFAULT_LOCK_TTL_HOURS), |s| {
                Duration::seconds(s)
            }),
        )
        .await?;

    Ok(Status::NoContent)
}
