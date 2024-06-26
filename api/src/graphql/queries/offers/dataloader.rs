use async_graphql::dataloader::Loader;
use entity::offer_details;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use std::{collections::HashMap, sync::Arc};

pub struct OfferDetailsLoader {
    pub database: DatabaseConnection,
}

impl Loader<i64> for OfferDetailsLoader {
    type Value = offer_details::Model;
    type Error = Arc<DbErr>;

    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        Ok(offer_details::Entity::find()
            .filter(offer_details::Column::PropositionId.is_in(keys.iter().copied()))
            .all(&self.database)
            .await
            .map_err(Arc::new)?
            .into_iter()
            .map(|o| (o.proposition_id, o))
            .collect::<HashMap<_, _>>())
    }
}
