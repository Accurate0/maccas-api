use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    Categories,
    Products,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .drop_column(OfferDetails::Products)
                    .add_column(
                        ColumnDef::new(OfferDetails::Categories)
                            .array(ColumnType::Text)
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .drop_column(OfferDetails::Categories)
                    .add_column(
                        ColumnDef::new(OfferDetails::Products)
                            .array(ColumnType::Text)
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }
}
