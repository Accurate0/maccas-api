use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Events {
    Table,
    Data,
    Hash,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .add_column(
                        ColumnDef::new(Events::Hash)
                            .string()
                            .generated(
                                Expr::cust_with_expr("md5($1 #>> '{}')", Expr::col(Events::Data)),
                                true,
                            )
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .drop_column(Events::Hash)
                    .to_owned(),
            )
            .await
    }
}
