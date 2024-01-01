use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Events {
    Table,
    Id,
    EventId,
    Name,
    CreatedAt,
    UpdatedAt,
    ShouldBeCompletedAt,
    IsCompleted,
    CompletedAt,
    Attempts,
    Error,
    ErrorMessage,
    Data,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Events::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Events::Name).string().not_null())
                    .col(
                        ColumnDef::new(Events::Id)
                            .integer()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Events::EventId)
                            .uuid()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Events::Data).json_binary().not_null())
                    .col(ColumnDef::new(Events::IsCompleted).boolean().not_null())
                    .col(
                        ColumnDef::new(Events::ShouldBeCompletedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Events::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Events::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Events::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Events::Error)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Events::ErrorMessage).string())
                    .col(ColumnDef::new(Events::CompletedAt).date_time().null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(r#"
            CREATE TRIGGER update_events_updated_at BEFORE UPDATE ON events FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
            "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS update_events_updated_at ON events")
            .await?;

        manager
            .drop_table(Table::drop().table(Events::Table).to_owned())
            .await
    }
}
