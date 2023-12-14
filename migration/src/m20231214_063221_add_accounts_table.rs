use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Name,
    LoginPassword,
    LoginUsername,
    AccessToken,
    RefreshToken,
    LastAccessed,
}

#[derive(DeriveIden)]
enum Offers {
    Table,
    AccountName,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Accounts::Name)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Accounts::LoginUsername).string().not_null())
                    .col(ColumnDef::new(Accounts::LoginPassword).string().not_null())
                    .col(ColumnDef::new(Accounts::AccessToken).string().not_null())
                    .col(ColumnDef::new(Accounts::RefreshToken).string().not_null())
                    .col(
                        ColumnDef::new(Accounts::LastAccessed)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .add_column(ColumnDef::new(Offers::AccountName).string().not_null())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("AccountName")
                            .from_tbl(Offers::Table)
                            .from_col(Offers::AccountName)
                            .to_tbl(Accounts::Table)
                            .to_col(Accounts::Name)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .drop_column(Offers::AccountName)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await
    }
}
