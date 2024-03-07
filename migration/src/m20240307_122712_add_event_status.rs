use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum EventStatus {
    #[sea_orm(iden = "event_status")]
    Type,
    Pending,
    Completed,
    Failed,
    Running,
}

#[derive(DeriveIden)]
enum Events {
    Table,
    Status,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(EventStatus::Type)
                    .values([
                        EventStatus::Completed,
                        EventStatus::Failed,
                        EventStatus::Pending,
                        EventStatus::Running,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .add_column(
                        ColumnDef::new(Events::Status)
                            .custom(EventStatus::Type)
                            .default(SimpleExpr::Value(Value::String(Some(Box::new(
                                EventStatus::Completed.to_string(),
                            )))))
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Events::Table)
                    .drop_column(Events::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_type(Type::drop().name(EventStatus::Type).to_owned())
            .await
    }
}
