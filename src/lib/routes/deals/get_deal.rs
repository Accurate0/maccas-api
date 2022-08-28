use crate::{
    routes::Context,
    types::{api::GetDealsOffer, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Information for specified deal", body = GetDealsOffer),
        (status = 404, description = "Deal not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[get("/deals/<deal_id>")]
pub async fn get_deal(
    ctx: &State<Context<'_>>,
    deal_id: &str,
) -> Result<Json<GetDealsOffer>, ApiError> {
    if let Ok((_, offer)) = ctx.database.get_offer_by_id(deal_id).await {
        Ok(Json(GetDealsOffer::from(offer)))
    } else {
        Err(ApiError::NotFound)
    }
}
