use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Offers {
    Table,
    OfferId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .modify_column(ColumnDef::new(Offers::OfferId).big_integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .modify_column(ColumnDef::new(Offers::OfferId).integer())
                    .to_owned(),
            )
            .await
    }
}
