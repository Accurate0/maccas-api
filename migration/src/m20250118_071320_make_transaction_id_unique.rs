use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferAudit {
    Table,
    TransactionId,
    CreatedAt,
}

const INDEX_NAME: &str = "offer_audit_transaction_id_unique";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(INDEX_NAME)
                    .table(OfferAudit::Table)
                    .col(OfferAudit::TransactionId)
                    .col(OfferAudit::CreatedAt)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
