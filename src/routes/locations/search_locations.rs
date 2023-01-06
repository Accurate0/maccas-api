use crate::{
    constants::{mc_donalds, LOCATION_SEARCH_DISTANCE},
    extensions::SecretsManagerExtensions,
    proxy, routes,
    types::{api::RestaurantInformation, error::ApiError},
};
use foundation::constants;
use foundation::constants::{CORRELATION_ID_HEADER, X_API_KEY_HEADER};
use foundation::rocket::guards::correlation_id::CorrelationId;
use foundation::types::places::PlacesResponse;
use http::Method;
use rocket::{serde::json::Json, State};

#[utoipa::path(
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
    let http_client = foundation::http::get_default_http_client();
    let response = http_client
        .request(
            Method::GET,
            format!("{}/place?text={}", constants::PLACES_API_BASE_URL, text,).as_str(),
        )
        .header(CORRELATION_ID_HEADER, correlation_id.0)
        .header(
            X_API_KEY_HEADER,
            &ctx.secrets_client.get_apim_api_key().await,
        )
        .send()
        .await?
        .json::<PlacesResponse>()
        .await?;

    let proxy = proxy::get_proxy(&ctx.config);
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
