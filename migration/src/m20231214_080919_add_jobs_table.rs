use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Id,
    Name,
    LastRun,
}

#[derive(DeriveIden)]
enum CurrentJobs {
    Table,
    Name,
    Id,
    ResumeContext,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Jobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Jobs::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Jobs::LastRun).date_time())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CurrentJobs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CurrentJobs::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(CurrentJobs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CurrentJobs::ResumeContext).json_binary())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("Id")
                            .from_tbl(Jobs::Table)
                            .from_col(Jobs::Id)
                            .to_tbl(CurrentJobs::Table)
                            .to_col(CurrentJobs::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jobs::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(CurrentJobs::Table).to_owned())
            .await
    }
}
