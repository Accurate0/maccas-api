use crate::{
    client,
    constants::mc_donalds,
    guards::protected::ProtectedRoute,
    routes,
    types::{
        api::{OfferPointsResponse, OfferResponse},
        error::ApiError,
    },
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Random code for account", body = OfferPointsResponse),
        (status = 404, description = "Account not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "points",
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, header, description = "Key to bypass JWT checks"),
    ),
)]
#[get("/points/<account_id>?<store>")]
pub async fn get_points_by_id(
    ctx: &State<routes::Context<'_>>,
    _protected: ProtectedRoute,
    account_id: &str,
    store: i64,
) -> Result<Json<OfferPointsResponse>, ApiError> {
    if let Ok((account, points)) = ctx.database.get_points_by_account_hash(account_id).await {
        let http_client = client::get_http_client();
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

        let response = api_client
            .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
            .await?;

        Ok(Json(OfferPointsResponse {
            offer_response: OfferResponse::from(response.body),
            points_response: points,
        }))
    } else {
        Err(ApiError::NotFound)
    }
}
