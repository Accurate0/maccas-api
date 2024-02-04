use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AccountLock {
    Table,
    Id,
    CreatedAt,
    UnlockAt,
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
                    .table(AccountLock::Table)
                    .col(
                        ColumnDef::new(AccountLock::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AccountLock::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AccountLock::UnlockAt).date_time().not_null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("account_id_fk")
                            .from_tbl(AccountLock::Table)
                            .from_col(AccountLock::Id)
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
        manager
            .drop_table(Table::drop().table(AccountLock::Table).to_owned())
            .await
    }
}
