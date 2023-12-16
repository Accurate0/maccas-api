use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Name,
    Id,
    ResumeContext,
    LastExecution,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Jobs::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Jobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Jobs::LastExecution).date_time())
                    .col(ColumnDef::new(Jobs::ResumeContext).json_binary())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await
    }
}
