use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferNameClusterAssociation {
    Table,
    Name,
    ClusterId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OfferNameClusterAssociation::Table)
                    .col(string(OfferNameClusterAssociation::Name).primary_key())
                    .col(big_integer(OfferNameClusterAssociation::ClusterId))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(OfferNameClusterAssociation::Table)
                    .to_owned(),
            )
            .await
    }
}
