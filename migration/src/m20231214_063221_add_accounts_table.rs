use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    Password,
    Username,
    AccessToken,
    RefreshToken,
    DeviceId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Offers {
    Table,
    AccountId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Accounts::Username)
                            .unique_key()
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Accounts::Password).string().not_null())
                    .col(ColumnDef::new(Accounts::AccessToken).string().not_null())
                    .col(ColumnDef::new(Accounts::RefreshToken).string().not_null())
                    .col(ColumnDef::new(Accounts::DeviceId).string().not_null())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .default(Expr::current_timestamp())
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Accounts::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();

        db.execute_unprepared(r#"
        CREATE TRIGGER update_accounts_updated_at_column BEFORE UPDATE ON accounts FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
        "#).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .add_column(ColumnDef::new(Offers::AccountId).uuid().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("AccountId")
                            .from_tbl(Offers::Table)
                            .from_col(Offers::AccountId)
                            .to_tbl(Accounts::Table)
                            .to_col(Accounts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "DROP TRIGGER IF EXISTS update_accounts_updated_at_column ON accounts",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .drop_column(Offers::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}
