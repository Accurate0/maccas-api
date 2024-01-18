use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum JobHistory {
    Table,
    Id,
    JobId,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
    Error,
    ErrorMessage,
    Context,
}

#[derive(DeriveIden)]
enum Jobs {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JobHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(JobHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(JobHistory::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(JobHistory::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(JobHistory::CompletedAt).date_time().null())
                    .col(
                        ColumnDef::new(JobHistory::Error)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(JobHistory::Context).json_binary().null())
                    .col(ColumnDef::new(JobHistory::ErrorMessage).string())
                    .col(ColumnDef::new(JobHistory::JobId).uuid().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("job_id_fk")
                            .to_tbl(Jobs::Table)
                            .to_col(Jobs::Id)
                            .from_tbl(JobHistory::Table)
                            .from_col(JobHistory::JobId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(r#"
                CREATE TRIGGER update_job_history_updated_at BEFORE UPDATE ON job_history FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
                "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP TRIGGER IF EXISTS update_job_history_updated_at ON job_history",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(JobHistory::Table).to_owned())
            .await
    }
}
