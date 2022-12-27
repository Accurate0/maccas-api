use crate::{
    constants::mc_donalds,
    routes,
    types::{api::OfferResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Random code for specified deal", body = OfferResponse),
        (status = 404, description = "Deal not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[get("/code/<deal_id>?<store>")]
pub async fn get_code(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: i64,
) -> Result<Json<OfferResponse>, ApiError> {
    if let Ok((account, _offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let http_client = foundation::http::get_http_client();
        let api_client = ctx
            .database
            .get_specific_client(
                &http_client,
                &ctx.config.mcdonalds.client_id,
                &ctx.config.mcdonalds.client_secret,
                &ctx.config.mcdonalds.sensor_data,
                &account,
                false,
            )
            .await?;

        let resp = api_client
            .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
            .await?;

        Ok(Json(OfferResponse::from(resp.body)))
    } else {
        Err(ApiError::NotFound)
    }
}
