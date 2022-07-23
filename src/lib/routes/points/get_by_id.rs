use crate::{
    client,
    constants::mc_donalds,
    routes,
    types::{
        api::{OfferPointsResponse, OfferResponse},
        error::ApiError,
    },
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    get,
    path = "/points/{accountId}",
    responses(
        (status = 200, description = "Random code for account", body = OfferPointsResponse),
        (status = 404, description = "Account not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    params(
        ("accountId" = String, path, description = "The account id"),
        ("store" = Option<i64>, query, description = "The selected store"),
    ),
    tag = "points",
)]
#[get("/points/<account_id>?<store>")]
pub async fn get_points_by_id(
    ctx: &State<routes::Context<'_>>,
    account_id: &str,
    store: Option<i64>,
) -> Result<Json<OfferPointsResponse>, ApiError> {
    if let Ok((account, points)) = ctx.database.get_points_by_account_hash(account_id).await {
        let http_client = client::get_http_client();
        let api_client = ctx
            .database
            .get_specific_client(
                &http_client,
                &ctx.config.client_id,
                &ctx.config.client_secret,
                &ctx.config.sensor_data,
                &account,
                false,
            )
            .await?;

        let response = api_client
            .get_offers_dealstack(
                mc_donalds::default::OFFSET,
                &store.unwrap_or(mc_donalds::default::STORE_ID),
            )
            .await?;

        Ok(Json(OfferPointsResponse {
            offer_response: OfferResponse::from(response.body),
            points_response: points,
        }))
    } else {
        Err(ApiError::NotFound)
    }
}
