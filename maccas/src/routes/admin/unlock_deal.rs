use crate::{
    database::offer::OfferRepository, guards::admin::AdminOnlyRoute, types::error::ApiError,
};
use rocket::{http::Status, State};

#[utoipa::path(
    responses(
        (status = 204, description = "Unlocked deal"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[delete("/admin/locked-deals/<deal_id>")]
pub async fn unlock_deal(
    offer_repo: &State<OfferRepository>,
    _admin: AdminOnlyRoute,
    deal_id: &str,
) -> Result<Status, ApiError> {
    offer_repo.unlock_offer(deal_id).await?;

    Ok(Status::NoContent)
}
