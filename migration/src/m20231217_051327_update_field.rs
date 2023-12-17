use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    Price,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .modify_column(ColumnDef::new(OfferDetails::Price).double().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .modify_column(ColumnDef::new(OfferDetails::Price).double().not_null())
                    .to_owned(),
            )
            .await
    }
}
