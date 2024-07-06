use entity::sea_orm_active_enums::EventStatus as CurrentEventStatus;
use extension::postgres::Type;
use sea_orm::ActiveEnum;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
pub enum EventStatus {
    Duplicate,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(CurrentEventStatus::name())
                    .add_value(EventStatus::Duplicate)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
