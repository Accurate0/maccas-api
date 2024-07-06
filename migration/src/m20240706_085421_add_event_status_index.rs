use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Events {
    Table,
    Status,
}

// This index helps the is completed table scan
// when reloading incomplete events
const INDEX_NAME: &str = "idx_events_status";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Events::Table)
                    .name(INDEX_NAME)
                    .col(Events::Status)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(INDEX_NAME)
                    .table(Events::Table)
                    .to_owned(),
            )
            .await
    }
}
