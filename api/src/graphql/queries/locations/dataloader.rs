use super::types::{DataloaderLocation, Location};
use crate::settings::Settings;
use async_graphql::dataloader::Loader;
use base::{
    constants::mc_donalds::{FILTER, LOCATION_SEARCH_DISTANCE, STORE_UNIQUE_ID_TYPE},
    maccas,
};
use entity::accounts;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use std::{collections::HashMap, sync::Arc};

pub struct LocationLoader {
    pub database: DatabaseConnection,
    pub settings: Settings,
}

#[async_trait::async_trait]
impl Loader<DataloaderLocation> for LocationLoader {
    type Value = Vec<Location>;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[DataloaderLocation],
    ) -> Result<HashMap<DataloaderLocation, Self::Value>, Self::Error> {
        // pick more recently updated account
        let account_to_use = accounts::Entity::find()
            .order_by_desc(accounts::Column::UpdatedAt)
            .one(&self.database)
            .await
            .map_err(|e| Arc::new(e.into()))?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let account_id = account_to_use.id.to_owned();
        tracing::info!("picked account: {:?}", &account_id);

        let proxy = reqwest::Proxy::all(self.settings.proxy.url.clone())
            .map_err(|e| Arc::new(e.into()))?
            .basic_auth(&self.settings.proxy.username, &self.settings.proxy.password);

        let api_client = maccas::get_activated_maccas_api_client(
            account_to_use,
            proxy,
            &self.settings.mcdonalds.client_id,
            &self.database,
        )
        .await?;

        // FIXME: parallel
        let mut location_map = HashMap::new();
        for loc in keys {
            let response = api_client
                .restaurant_location(&LOCATION_SEARCH_DISTANCE, &loc.lat, &loc.long, FILTER)
                .await;

            match response {
                Ok(v) => {
                    location_map.insert(loc, v);
                }
                Err(e) => tracing::error!("error fetching location: {}", e),
            };
        }

        Ok(location_map
            .into_iter()
            .map(|(loc, resp)| (loc, resp.body.response))
            .map(|(loc, restaurants)| match restaurants {
                Some(r) => (
                    loc.clone(),
                    r.restaurants
                        .iter()
                        .map(|r| Location {
                            name: r.name.clone(),
                            store_number: r.national_store_number.to_string(),
                            address: r.address.address_line1.clone(),
                        })
                        .collect::<Vec<_>>(),
                ),
                None => (loc.clone(), vec![]),
            })
            .collect::<HashMap<_, _>>())
    }
}

#[async_trait::async_trait]
impl Loader<String> for LocationLoader {
    type Value = Location;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        // pick more recently updated account
        let account_to_use = accounts::Entity::find()
            .order_by_desc(accounts::Column::UpdatedAt)
            .one(&self.database)
            .await
            .map_err(|e| Arc::new(e.into()))?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let account_id = account_to_use.id.to_owned();
        tracing::info!("picked account: {:?}", &account_id);

        let proxy = reqwest::Proxy::all(self.settings.proxy.url.clone())
            .map_err(|e| Arc::new(e.into()))?
            .basic_auth(&self.settings.proxy.username, &self.settings.proxy.password);

        let api_client = maccas::get_activated_maccas_api_client(
            account_to_use,
            proxy,
            &self.settings.mcdonalds.client_id,
            &self.database,
        )
        .await?;

        Ok(futures::future::try_join_all(
            keys.iter()
                .map(|store_id| api_client.get_restaurant(store_id, FILTER, STORE_UNIQUE_ID_TYPE)),
        )
        .await
        .into_iter()
        .flatten()
        .filter_map(|r| r.body.response)
        .map(|inner| {
            (
                inner.restaurant.national_store_number.to_string(),
                Location {
                    name: inner.restaurant.name,
                    store_number: inner.restaurant.national_store_number.to_string(),
                    address: inner.restaurant.address.address_line1,
                },
            )
        })
        .collect::<HashMap<_, _>>())
    }
}
