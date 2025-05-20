use self::types::{
    CoordinateSearchInput, Location, QueriedLocation, StoreIdInput, TextSearchInput,
};
use crate::graphql::queries::locations::dataloader::LocationLoader;
use crate::graphql::queries::locations::types::LocationRequest;
use crate::settings::Settings;
use async_graphql::dataloader::DataLoader;
use async_graphql::{Context, Object};
use entity::stores;
use places::types::Location as PlaceLocation;
use places::types::{Area, PlacesRequest, Rectangle};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

pub mod dataloader;
mod types;

#[derive(Default)]
pub struct LocationsQuery;

#[Object]
impl LocationsQuery {
    async fn location<'a>(&self) -> Result<QueriedLocation, anyhow::Error> {
        Ok(QueriedLocation {})
    }
}

// TODO: implement caching for locations (redis?)

#[Object]
impl QueriedLocation {
    async fn text<'a>(
        &self,
        ctx: &Context<'a>,
        input: TextSearchInput,
    ) -> async_graphql::Result<Vec<Location>> {
        let settings = ctx.data::<Settings>()?;
        let http_client = ctx.data::<ClientWithMiddleware>()?.clone();
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

        match places_response.body.places.first() {
            Some(response) => {
                let loader = ctx.data::<DataLoader<LocationLoader>>()?;
                match loader
                    .load_one(LocationRequest {
                        lat: response.location.latitude,
                        long: response.location.longitude,
                    })
                    .await?
                {
                    Some(v) => Ok(v),
                    None => Ok(vec![]),
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
        let loader = ctx.data::<DataLoader<LocationLoader>>()?;
        match loader
            .load_one(LocationRequest {
                lat: input.lat,
                long: input.lng,
            })
            .await?
        {
            Some(v) => Ok(v),
            None => Ok(vec![]),
        }
    }

    async fn store_id<'a>(
        &self,
        ctx: &Context<'a>,
        input: StoreIdInput,
    ) -> async_graphql::Result<Location> {
        let db = ctx.data::<DatabaseConnection>()?;

        match stores::Entity::find_by_id(&input.store_id).one(db).await? {
            Some(model) => Ok(Location {
                name: model.name,
                store_number: model.id,
                address: model.address,
                distance: None,
            }),
            None => {
                let loader = ctx.data::<DataLoader<LocationLoader>>()?;
                match loader.load_one(input.store_id).await? {
                    Some(l) => {
                        stores::ActiveModel {
                            id: Set(l.store_number.clone()),
                            name: Set(l.name.clone()),
                            address: Set(l.address.clone()),
                            ..Default::default()
                        }
                        .insert(db)
                        .await?;

                        Ok(l)
                    }
                    None => Err(anyhow::Error::msg("No store found for specified id").into()),
                }
            }
        }
    }
}
