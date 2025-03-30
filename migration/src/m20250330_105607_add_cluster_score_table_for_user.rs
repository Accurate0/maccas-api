use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferClusterScore {
    Table,
    Id,
    UserId,
    ClusterId,
    Score,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OfferClusterScore::Table)
                    .if_not_exists()
                    .col(pk_auto(OfferClusterScore::Id))
                    .col(uuid(OfferClusterScore::UserId))
                    .col(big_integer(OfferClusterScore::ClusterId))
                    .col(double(OfferClusterScore::Score))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OfferClusterScore::Table).to_owned())
            .await
    }
}
