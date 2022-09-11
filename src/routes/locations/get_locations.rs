use crate::{
    client,
    constants::mc_donalds,
    routes,
    types::{api::RestaurantInformation, error::ApiError},
};
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
    let http_client = client::get_http_client();
    let api_client = ctx
        .database
        .get_specific_client(
            &http_client,
            &ctx.config.mcdonalds.client_id,
            &ctx.config.mcdonalds.client_secret,
            &ctx.config.mcdonalds.sensor_data,
            &ctx.config.mcdonalds.service_account,
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
