use self::types::{CoordinateSearchInput, Location, TextSearchInput};
use crate::{
    constants::{
        config::CONFIG_PLACES_API_KEY_ID,
        mc_donalds::{self, default::LOCATION_SEARCH_DISTANCE},
    },
    proxy,
    routes::Context,
    types::api::RestaurantInformation,
};
use async_graphql::Object;
use foundation::extensions::SecretsManagerExtensions;
use places::types::Location as PlaceLocation;
use places::types::{Area, PlacesRequest, Rectangle};

mod types;

#[derive(Default)]
pub struct LocationsQuery;

#[Object]
impl LocationsQuery {
    async fn location<'a>(&self) -> Result<Location, anyhow::Error> {
        Ok(Location {})
    }
}

#[Object]
impl Location {
    async fn text<'ctx>(
        &self,
        gql_ctx: &async_graphql::Context<'ctx>,
        input: TextSearchInput,
    ) -> Result<Vec<RestaurantInformation>, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
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
                text_query: input.query,
                max_result_count: 1,
                location_bias: Area {
                    rectangle: Rectangle {
                        low: PlaceLocation {
                            latitude: -46.2858922444765,
                            longitude: 109.62638287960314,
                        },
                        high: PlaceLocation {
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

        let account = &ctx
            .database
            .account_repository
            .get_user_account(&ctx.config.mcdonalds.service_account_name)
            .await?;

        let api_client = ctx
            .database
            .account_repository
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
                        let mut location_list = Vec::new();
                        for restaurant in response.restaurants {
                            location_list.push(RestaurantInformation::from(restaurant));
                        }
                        Ok(location_list)
                    }
                    _ => Ok(vec![]),
                }
            }
            None => Ok(vec![]),
        }
    }

    async fn coordinates<'ctx>(
        &self,
        gql_ctx: &async_graphql::Context<'ctx>,
        input: CoordinateSearchInput,
    ) -> Result<Vec<RestaurantInformation>, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        let proxy = proxy::get_proxy(&ctx.config.proxy).await;
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let account = &ctx
            .database
            .account_repository
            .get_user_account(&ctx.config.mcdonalds.service_account_name)
            .await?;

        let api_client = ctx
            .database
            .account_repository
            .get_api_client(
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
                &input.distance,
                &input.lat,
                &input.lng,
                mc_donalds::default::FILTER,
            )
            .await?;

        let response = resp.body.response;
        match response {
            Some(response) if !response.restaurants.is_empty() => {
                let mut location_list = Vec::new();
                for restaurant in response.restaurants {
                    location_list.push(RestaurantInformation::from(restaurant));
                }
                Ok(location_list)
            }
            _ => Ok(vec![]),
        }
    }
}
