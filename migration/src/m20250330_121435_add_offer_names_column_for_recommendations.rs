use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Recommendations {
    Table,
    Names,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Recommendations::Table)
                    .add_column(
                        array(Recommendations::Names, ColumnType::Text)
                            .default(Expr::expr(SimpleExpr::Custom("'{}'".to_owned()))),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Recommendations::Table)
                    .drop_column(Recommendations::Names)
                    .to_owned(),
            )
            .await
    }
}
