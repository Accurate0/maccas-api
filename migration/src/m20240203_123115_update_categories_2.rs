use entity::categories;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EntityTrait, Set, TransactionTrait},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let categories = ["Fries"];

        let txn = db.begin().await?;
        let category_models = categories.map(|c| categories::ActiveModel {
            name: Set(c.to_owned()),
            ..Default::default()
        });

        categories::Entity::insert_many(category_models)
            .on_conflict(
                OnConflict::column(categories::Column::Name)
                    .do_nothing()
                    .to_owned(),
            )
            .exec_without_returning(&txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
