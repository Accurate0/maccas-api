use crate::{
    constants::mc_donalds,
    proxy, routes,
    types::{api::RestaurantInformationList, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of locations near specified coordinates", body = RestaurantInformationList),
        (status = 400, description = "Invalid/missing parameters"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "location",
)]
#[get("/locations?<distance>&<latitude>&<longitude>")]
pub async fn get_locations(
    ctx: &State<routes::Context<'_>>,
    distance: f64,
    latitude: f64,
    longitude: f64,
) -> Result<Json<RestaurantInformationList>, ApiError> {
    let proxy = proxy::get_proxy(&ctx.config.proxy).await;
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
    let account = &ctx
        .database
        .get_account(&ctx.config.mcdonalds.service_account_name)
        .await?;

    let api_client = ctx
        .database
        .get_specific_client(
            http_client,
            &ctx.config.mcdonalds.client_id,
            &ctx.config.mcdonalds.client_secret,
            &ctx.config.mcdonalds.sensor_data,
            account,
            false,
        )
        .await?;
    let resp = api_client
        .restaurant_location(
            &distance,
            &latitude,
            &longitude,
            mc_donalds::default::FILTER,
        )
        .await?;

    let response = resp.body.response;

    Ok(Json(RestaurantInformationList::from(response)))
}
