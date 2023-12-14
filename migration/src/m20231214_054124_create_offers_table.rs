use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Offers {
    Table,
    Id,
    OfferId,
    ValidFrom,
    ValidTo,
    Name,
    ShortName,
    Description,
    CreationDate,
    ImageBaseName,
    OriginalImageBaseName,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Offers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Offers::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Offers::OfferId).integer().not_null())
                    .col(ColumnDef::new(Offers::ValidFrom).date_time().not_null())
                    .col(ColumnDef::new(Offers::ValidTo).date_time().not_null())
                    .col(ColumnDef::new(Offers::Name).string().not_null())
                    .col(ColumnDef::new(Offers::ShortName).string().not_null())
                    .col(ColumnDef::new(Offers::Description).string().not_null())
                    .col(ColumnDef::new(Offers::CreationDate).date_time().not_null())
                    .col(ColumnDef::new(Offers::ImageBaseName).string().not_null())
                    .col(
                        ColumnDef::new(Offers::OriginalImageBaseName)
                            .string()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Offers::Table).to_owned())
            .await
    }
}
