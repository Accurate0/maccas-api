use self::types::{CoordinateSearchInput, Location, QueriedLocation, TextSearchInput};
use crate::settings::Settings;
use async_graphql::{Context, Object};
use base::constants::mc_donalds::{FILTER, LOCATION_SEARCH_DISTANCE};
use base::http::get_http_client;
use entity::accounts;
use places::types::Location as PlaceLocation;
use places::types::{Area, PlacesRequest, Rectangle};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryOrder, Set,
};

mod types;

#[derive(Default)]
pub struct LocationsQuery;

#[Object]
impl LocationsQuery {
    async fn location<'a>(&self) -> Result<QueriedLocation, anyhow::Error> {
        Ok(QueriedLocation {})
    }
}

// FIXME: move common logic for getting "activated" api client for specific account
// FIXME: don't refresh accounts within 14 mins
// FIXME: appoint service account

#[Object]
impl QueriedLocation {
    async fn text<'a>(
        &self,
        ctx: &Context<'a>,
        input: TextSearchInput,
    ) -> async_graphql::Result<Vec<Location>> {
        let settings = ctx.data::<Settings>()?;
        let db = ctx.data::<DatabaseConnection>()?;

        let http_client = base::http::get_simple_http_client()?;
        let api_client = places::ApiClient::new(settings.places_api_key.clone(), http_client);

        // Australia square, low -> high
        // -46.2858922444765, 109.62638287960314
        // -10.481731180947541, 156.54739153571109
        let places_response = api_client
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

        tracing::info!("locations found: {:?}", places_response.body);

        let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
            .basic_auth(&settings.proxy.username, &settings.proxy.password);

        // pick more recently updated account
        let account_to_use = accounts::Entity::find()
            .order_by_desc(accounts::Column::UpdatedAt)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let account_id = account_to_use.id.to_owned();
        tracing::info!("picked account: {:?}", &account_id);

        let mut api_client = libmaccas::ApiClient::new(
            base::constants::mc_donalds::BASE_URL.to_owned(),
            get_http_client(proxy)?,
            settings.mcdonalds.client_id.clone(),
        );

        api_client.set_auth_token(&account_to_use.access_token);
        let response = api_client
            .customer_login_refresh(&account_to_use.refresh_token)
            .await?;

        let response = response
            .body
            .response
            .ok_or_else(|| anyhow::Error::msg("access token refresh failed"))?;

        api_client.set_auth_token(&response.access_token);

        let mut update_model = account_to_use.into_active_model();

        update_model.access_token = Set(response.access_token);
        update_model.refresh_token = Set(response.refresh_token);
        tracing::info!("new tokens fetched, updating database");

        update_model.update(db).await?;

        match places_response.body.places.first() {
            Some(response) => {
                let lat = response.location.latitude;
                let lng = response.location.longitude;
                let response = api_client
                    .restaurant_location(&LOCATION_SEARCH_DISTANCE, &lat, &lng, FILTER)
                    .await?;

                match response.body.response {
                    Some(response) if !response.restaurants.is_empty() => Ok(response
                        .restaurants
                        .into_iter()
                        .map(|r| Location {
                            name: r.name,
                            store_number: r.national_store_number,
                            address: r.address.address_line1,
                        })
                        .collect()),
                    _ => Ok(vec![]),
                }
            }
            None => Ok(vec![]),
        }
    }

    async fn coordinate<'a>(
        &self,
        ctx: &Context<'a>,
        input: CoordinateSearchInput,
    ) -> async_graphql::Result<Vec<Location>> {
        let settings = ctx.data::<Settings>()?;
        let db = ctx.data::<DatabaseConnection>()?;

        let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
            .basic_auth(&settings.proxy.username, &settings.proxy.password);

        // pick more recently updated account
        let account_to_use = accounts::Entity::find()
            .order_by_desc(accounts::Column::UpdatedAt)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let account_id = account_to_use.id.to_owned();
        tracing::info!("picked account: {:?}", &account_id);

        let mut api_client = libmaccas::ApiClient::new(
            base::constants::mc_donalds::BASE_URL.to_owned(),
            get_http_client(proxy)?,
            settings.mcdonalds.client_id.clone(),
        );

        api_client.set_auth_token(&account_to_use.access_token);
        let response = api_client
            .customer_login_refresh(&account_to_use.refresh_token)
            .await?;

        let response = response
            .body
            .response
            .ok_or_else(|| anyhow::Error::msg("access token refresh failed"))?;

        api_client.set_auth_token(&response.access_token);

        let mut update_model = account_to_use.into_active_model();

        update_model.access_token = Set(response.access_token);
        update_model.refresh_token = Set(response.refresh_token);
        tracing::info!("new tokens fetched, updating database");

        update_model.update(db).await?;

        let response = api_client
            .restaurant_location(&input.distance, &input.lat, &input.lng, FILTER)
            .await?;

        match response.body.response {
            Some(response) if !response.restaurants.is_empty() => Ok(response
                .restaurants
                .into_iter()
                .map(|r| Location {
                    name: r.name,
                    store_number: r.national_store_number,
                    address: r.address.address_line1,
                })
                .collect()),
            _ => Ok(vec![]),
        }
    }
}
