use async_graphql::dataloader::Loader;
use entity::{offer_details, products};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::{collections::HashMap, sync::Arc};

pub struct OfferDetailsLoader {
    pub database: DatabaseConnection,
}

#[async_trait::async_trait]
impl Loader<i64> for OfferDetailsLoader {
    type Value = offer_details::Model;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        Ok(offer_details::Entity::find()
            .filter(offer_details::Column::PropositionId.is_in(keys.iter().copied()))
            .all(&self.database)
            .await
            .map_err(Arc::new)
            .unwrap()
            .into_iter()
            .map(|o| (o.proposition_id, o))
            .collect::<HashMap<_, _>>())
    }
}

pub struct ProductLoader {
    pub database: DatabaseConnection,
}

#[async_trait::async_trait]
impl Loader<i64> for ProductLoader {
    type Value = products::Model;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        Ok(products::Entity::find()
            .filter(products::Column::Id.is_in(keys.iter().copied()))
            .all(&self.database)
            .await
            .map_err(Arc::new)
            .unwrap()
            .into_iter()
            .map(|o| (o.id, o))
            .collect::<HashMap<_, _>>())
    }
}
