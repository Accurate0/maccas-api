use crate::{graphql::queries::offers::types::OfferCount, name_of};
use async_graphql::dataloader::Loader;
use entity::{offer_details, offers};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait,
};
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

pub struct OfferCountDataLoader {
    pub database: DatabaseConnection,
}

impl Loader<String> for OfferCountDataLoader {
    type Value = i64;
    type Error = Arc<DbErr>;

    async fn load(&self, names: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let db = &self.database;
        let all_locked_accounts = entity::account_lock::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.id);

        let mut conditions = Condition::all();
        for locked_account in all_locked_accounts {
            conditions = conditions.add(offers::Column::AccountId.ne(locked_account));
        }

        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = conditions
            .add(offers::Column::ValidTo.gt(now))
            .add(offers::Column::ValidFrom.lt(now))
            .add(offer_details::Column::ShortName.is_in(names));

        Ok(offers::Entity::find()
            .select_only()
            .filter(conditions.clone())
            .join(JoinType::InnerJoin, offers::Relation::OfferDetails.def())
            .column(offer_details::Column::ShortName)
            .column_as(
                offer_details::Column::ShortName.count(),
                name_of!(count in OfferCount),
            )
            .group_by(offer_details::Column::ShortName)
            .into_model::<OfferCount>()
            .all(db)
            .await?
            .into_iter()
            .map(|o| (o.short_name, o.count))
            .collect::<HashMap<_, _>>())
    }
}
