use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Jobs {
    Table,
    ResumeContext,
    Context,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Jobs::Table)
                    .rename_column(Jobs::ResumeContext, Jobs::Context)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Jobs::Table)
                    .rename_column(Jobs::Context, Jobs::ResumeContext)
                    .to_owned(),
            )
            .await
    }
}
