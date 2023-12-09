use crate::{
    constants::{
        config::CONFIG_PLACES_API_KEY_ID,
        mc_donalds::{self, default::LOCATION_SEARCH_DISTANCE},
    },
    database::account::AccountRepository,
    proxy, routes,
    types::{api::RestaurantInformationList, error::ApiError},
};
use foundation::extensions::SecretsManagerExtensions;
use places::types::{Area, Location, PlacesRequest, Rectangle};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Closest location near specified text", body = RestaurantInformationList),
        (status = 404, description = "No locations found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "location",
)]
#[get("/locations/search?<text>")]
pub async fn search_locations(
    ctx: &State<routes::Context>,
    account_repo: &State<AccountRepository>,
    text: &str,
) -> Result<Json<RestaurantInformationList>, ApiError> {
    let http_client = foundation::http::get_default_http_client();
    let api_client = places::ApiClient::new(
        ctx.secrets_client
            .get_secret(CONFIG_PLACES_API_KEY_ID)
            .await?,
        http_client,
    );

    // Australia square, low -> high
    // -46.2858922444765, 109.62638287960314
    // -10.481731180947541, 156.54739153571109
    let response = api_client
        .get_place_by_text(&PlacesRequest {
            text_query: text.to_owned(),
            max_result_count: 1,
            location_bias: Area {
                rectangle: Rectangle {
                    low: Location {
                        latitude: -46.2858922444765,
                        longitude: 109.62638287960314,
                    },
                    high: Location {
                        latitude: -10.481731180947541,
                        longitude: 156.54739153571109,
                    },
                },
            },
        })
        .await?;

    log::info!("locations found: {:?}", response.body);

    let proxy = proxy::get_proxy(&ctx.config.proxy).await;
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);

    let account = &account_repo
        .get_user_account(&ctx.config.mcdonalds.service_account_name)
        .await?;

    let api_client = account_repo
        .get_api_client(
            http_client,
            &ctx.config.mcdonalds.client_id,
            &ctx.config.mcdonalds.client_secret,
            &ctx.config.mcdonalds.sensor_data,
            account,
            false,
        )
        .await?;

    match response.body.places.first() {
        Some(response) => {
            let lat = response.location.latitude;
            let lng = response.location.longitude;
            let response = api_client
                .restaurant_location(
                    &LOCATION_SEARCH_DISTANCE,
                    &lat,
                    &lng,
                    mc_donalds::default::FILTER,
                )
                .await?;

            match response.body.response {
                Some(response) if !response.restaurants.is_empty() => {
                    Ok(Json(RestaurantInformationList::from(response)))
                }
                _ => Err(ApiError::NotFound),
            }
        }
        None => Err(ApiError::NotFound),
    }
}
