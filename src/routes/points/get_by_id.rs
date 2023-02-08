use crate::{
    constants::{config::MAX_PROXY_COUNT, mc_donalds},
    guards::protected::ProtectedRoute,
    proxy,
    rng::RNG,
    routes,
    types::{api::OfferPointsResponse, error::ApiError},
};
use rand::Rng;
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Random code for account", body = OfferPointsResponse),
        (status = 404, description = "Account not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "points",
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, Header, description = "Key to bypass JWT checks"),
    ),
)]
#[get("/points/<account_id>?<store>")]
pub async fn get_points_by_id(
    ctx: &State<routes::Context<'_>>,
    _protected: ProtectedRoute,
    account_id: &str,
    store: String,
) -> Result<Json<OfferPointsResponse>, ApiError> {
    if let Ok((account, points)) = ctx.database.get_points_by_account_hash(account_id).await {
        let mut rng = RNG.lock().await;
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

        let response = api_client
            .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
            .await?;

        Ok(Json(OfferPointsResponse {
            offer_response: response.body.into(),
            points_response: points.into(),
        }))
    } else {
        Err(ApiError::NotFound)
    }
}
