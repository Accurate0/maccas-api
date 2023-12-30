use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Stores {
    Table,
    Id,
    Name,
    Address,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Stores::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Stores::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Stores::Name).string().not_null())
                    .col(ColumnDef::new(Stores::Address).string().not_null())
                    .col(
                        ColumnDef::new(Stores::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Stores::Table).to_owned())
            .await
    }
}
