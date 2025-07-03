use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum ConcurrentActiveDeals {
    Table,
    UserId,
    Count,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConcurrentActiveDeals::Table)
                    .if_not_exists()
                    .col(uuid(ConcurrentActiveDeals::UserId).primary_key())
                    .col(integer(ConcurrentActiveDeals::Count))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ConcurrentActiveDeals::Table).to_owned())
            .await
    }
}
