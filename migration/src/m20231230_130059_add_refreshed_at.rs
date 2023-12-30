use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    RefreshedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .add_column(
                        ColumnDef::new(Accounts::RefreshedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE OR REPLACE FUNCTION accounts_update_refreshed_at_column()
                    RETURNS TRIGGER AS $$
                        BEGIN
                            NEW.refreshed_at = now();
                            RETURN NEW;
                        END;
                    $$ language 'plpgsql';"#,
        )
        .await?;
        db.execute_unprepared(
            r#"CREATE TRIGGER tr_update_refreshed_at
            BEFORE UPDATE OF access_token, refresh_token ON accounts
            FOR EACH ROW EXECUTE PROCEDURE accounts_update_refreshed_at_column();"#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP TRIGGER IF EXISTS tr_update_refreshed_at ON accounts")
            .await?;

        db.execute_unprepared("DROP FUNCTION IF EXISTS accounts_update_refreshed_at_column")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Accounts::Table)
                    .drop_column(Accounts::RefreshedAt)
                    .to_owned(),
            )
            .await
    }
}
