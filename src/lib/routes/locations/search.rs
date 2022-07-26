use crate::{
    client,
    constants::{self, api_base, mc_donalds, LOCATION_SEARCH_DISTANCE},
    guards::correlation_id::CorrelationId,
    routes,
    types::{api::RestaurantInformation, error::ApiError, places::PlaceResponse},
};
use http::Method;
use rocket::{serde::json::Json, State};

#[utoipa::path(
    get,
    path = "/locations/search",
    responses(
        (status = 200, description = "Closest location near specified text", body = RestaurantInformation),
        (status = 404, description = "No locations found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "location",
)]
#[get("/locations/search?<text>")]
pub async fn search_locations(
    ctx: &State<routes::Context<'_>>,
    text: &str,
    correlation_id: CorrelationId,
) -> Result<Json<RestaurantInformation>, ApiError> {
    let http_client = client::get_http_client();

    let response = http_client
        .request(
            Method::GET,
            format!("{}/place?text={}", api_base::PLACES, text,).as_str(),
        )
        .header(constants::CORRELATION_ID_HEADER, correlation_id.0)
        .header(constants::X_API_KEY_HEADER, &ctx.config.api_key)
        .send()
        .await?
        .json::<PlaceResponse>()
        .await?;

    let api_client = ctx
        .database
        .get_specific_client(
            &http_client,
            &ctx.config.client_id,
            &ctx.config.client_secret,
            &ctx.config.sensor_data,
            &ctx.config.service_account,
            false,
        )
        .await?;
    let response = response.result;

    match response {
        Some(response) => {
            let lat = response.geometry.location.lat;
            let lng = response.geometry.location.lng;
            let resp = api_client
                .restaurant_location(
                    &LOCATION_SEARCH_DISTANCE,
                    &lat,
                    &lng,
                    mc_donalds::default::FILTER,
                )
                .await?;

            match resp.body.response {
                Some(list) => {
                    let resp = list.restaurants.first();
                    match resp {
                        Some(resp) => Ok(Json(RestaurantInformation::from(resp.clone()))),
                        None => Err(ApiError::NotFound),
                    }
                }
                None => Err(ApiError::NotFound),
            }
        }
        None => Err(ApiError::NotFound),
    }
}
