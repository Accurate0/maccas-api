use crate::{
    constants::{config::MAX_PROXY_COUNT, mc_donalds},
    proxy, routes,
    types::{api::OfferResponse, error::ApiError},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
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
    store: String,
) -> Result<Json<OfferResponse>, ApiError> {
    if let Ok((account, _offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let mut rng = StdRng::from_entropy();
        let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);

        let proxy = proxy::get_proxy(&ctx.config, random_number);
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = ctx
            .database
            .get_specific_client(
                http_client,
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
