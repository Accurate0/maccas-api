use entity::categories;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{EntityTrait, Set, TransactionTrait},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let categories = ["Breakfast", "Drinks", "Desserts", "Meal", "Burger"];

        manager
            .truncate_table(Table::truncate().table(Categories::Table).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .modify_column(ColumnDef::new(Categories::Name).not_null().unique_key())
                    .modify_column(
                        ColumnDef::new(Categories::Id)
                            .not_null()
                            .integer()
                            .auto_increment(),
                    )
                    .to_owned(),
            )
            .await?;

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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .truncate_table(Table::truncate().table(Categories::Table).to_owned())
            .await?;

        Ok(())
    }
}
