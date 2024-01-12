use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Points {
    Table,
    AccountId,
    #[allow(clippy::enum_variant_names)]
    CurrentPoints,
    #[allow(clippy::enum_variant_names)]
    LifetimePoints,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Points::Table)
                    .col(
                        ColumnDef::new(Points::AccountId)
                            .primary_key()
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Points::CurrentPoints)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Points::LifetimePoints)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Points::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Points::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("AccountId")
                            .from_tbl(Points::Table)
                            .from_col(Points::AccountId)
                            .to_tbl(Accounts::Table)
                            .to_col(Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(r#"
                    CREATE TRIGGER update_points_updated_at_column BEFORE UPDATE ON points FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
                    "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS update_points_updated_at_column ON points")
            .await?;

        manager
            .drop_table(Table::drop().table(Points::Table).to_owned())
            .await
    }
}
