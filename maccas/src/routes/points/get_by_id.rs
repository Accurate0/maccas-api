use crate::{
    constants::mc_donalds,
    database::{account::AccountRepository, point::PointRepository},
    guards::protected::ProtectedRoute,
    proxy, routes,
    types::{api::OfferPointsResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Random code for account", body = OfferPointsResponse),
        (status = 404, description = "Account not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "points",
)]
#[get("/points/<account_id>?<store>")]
pub async fn get_points_by_id(
    _protected: ProtectedRoute,
    ctx: &State<routes::Context>,
    account_repo: &State<AccountRepository>,
    point_repo: &State<PointRepository>,
    account_id: &str,
    store: String,
) -> Result<Json<OfferPointsResponse>, ApiError> {
    if let Ok((account, points)) = point_repo.get_points_by_account_hash(account_id).await {
        let proxy = proxy::get_proxy(&ctx.config.proxy).await;
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = account_repo
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
