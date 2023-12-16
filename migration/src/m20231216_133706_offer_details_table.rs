use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    PropositionId,
    Name,
    Description,
    Price,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OfferDetails::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OfferDetails::PropositionId)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OfferDetails::Name).string().not_null())
                    .col(
                        ColumnDef::new(OfferDetails::Description)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(OfferDetails::Price).double().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OfferDetails::Table).to_owned())
            .await
    }
}
