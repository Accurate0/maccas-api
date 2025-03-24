use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Recommendations {
    Table,
    Id,
    UserId,
    // This is dumb but let's do it for now
    OfferPropositionIds,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Recommendations::Table)
                    .col(pk_auto(Recommendations::Id))
                    .col(uuid_uniq(Recommendations::UserId))
                    .col(array(
                        Recommendations::OfferPropositionIds,
                        ColumnType::BigUnsigned,
                    ))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Recommendations::Table).to_owned())
            .await
    }
}
