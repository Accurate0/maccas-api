use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Offers {
    Table,
    Name,
    Description,
    ShortName,
    ImageBaseName,
    OriginalImageBaseName,
}

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    ShortName,
    ImageBaseName,
    OriginalImageBaseName,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .drop_column(Offers::Name)
                    .drop_column(Offers::Description)
                    .drop_column(Offers::ShortName)
                    .drop_column(Offers::ImageBaseName)
                    .drop_column(Offers::OriginalImageBaseName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .add_column(ColumnDef::new(OfferDetails::ShortName).string().not_null())
                    .add_column(
                        ColumnDef::new(OfferDetails::ImageBaseName)
                            .string()
                            .not_null(),
                    )
                    .add_column(
                        ColumnDef::new(OfferDetails::OriginalImageBaseName)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .add_column_if_not_exists(ColumnDef::new(Offers::Name).string().not_null())
                    .add_column_if_not_exists(ColumnDef::new(Offers::ShortName).string().not_null())
                    .add_column_if_not_exists(
                        ColumnDef::new(Offers::Description).string().not_null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Offers::ImageBaseName).string().not_null(),
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(Offers::OriginalImageBaseName)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .drop_column(OfferDetails::ShortName)
                    .drop_column(OfferDetails::ImageBaseName)
                    .drop_column(OfferDetails::OriginalImageBaseName)
                    .to_owned(),
            )
            .await
    }
}
