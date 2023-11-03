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
    let response = api_client.get_place_by_text(text).await?;
    let proxy = proxy::get_proxy(&ctx.config.proxy).await;
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);

    let account = &account_repo
        .get_account(&ctx.config.mcdonalds.service_account_name)
        .await?;

    let api_client = account_repo
        .get_specific_client(
            http_client,
            &ctx.config.mcdonalds.client_id,
            &ctx.config.mcdonalds.client_secret,
            &ctx.config.mcdonalds.sensor_data,
            account,
            false,
        )
        .await?;

    log::info!("locations found: {:#?}", response.body);
    let response = response.body.candidates.first();

    match response {
        Some(response) => {
            let lat = response.geometry.location.lat;
            let lng = response.geometry.location.lng;
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
