use crate::{
    constants::{config::MAX_PROXY_COUNT, mc_donalds},
    proxy, routes,
    types::{api::RestaurantInformation, error::ApiError},
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of locations near specified coordinates", body = [RestaurantInformation]),
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
) -> Result<Json<Vec<RestaurantInformation>>, ApiError> {
    let mut rng = StdRng::from_entropy();
    let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);

    let proxy = proxy::get_proxy(&ctx.config, random_number);
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
    let account = &ctx.config.mcdonalds.service_account;
    let account = &account.into();
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

    let mut location_list = Vec::new();
    let response = resp.body.response;
    if let Some(response) = response {
        for restaurant in response.restaurants {
            location_list.push(RestaurantInformation::from(restaurant));
        }
    }

    Ok(Json(location_list))
}
