use sea_orm_migration::{prelude::*, schema::date_time};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Recommendations {
    Table,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Recommendations::Table)
                    .add_column(
                        date_time(Recommendations::CreatedAt).default(Expr::current_timestamp()),
                    )
                    .add_column(
                        date_time(Recommendations::UpdatedAt).default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(r#"
                    CREATE TRIGGER update_recommendations_updated_at BEFORE UPDATE ON recommendations FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
                    "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP TRIGGER IF EXISTS update_recommendations_updated_at ON recommendations",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Recommendations::Table)
                    .drop_column(Recommendations::CreatedAt)
                    .drop_column(Recommendations::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}
