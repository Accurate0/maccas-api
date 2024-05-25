use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

const ACCOUNTS_INDEX_NAME: &str = "idx_accounts_refreshfailurecount";
#[derive(DeriveIden)]
enum Accounts {
    Table,
    RefreshFailureCount,
}

const OFFERS_INDEX_NAME: &str = "idx_offers_accountid";
#[derive(DeriveIden)]
enum Offers {
    Table,
    AccountId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .table(Accounts::Table)
                    .col(Accounts::RefreshFailureCount)
                    .name(ACCOUNTS_INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Offers::Table)
                    .col(Offers::AccountId)
                    .name(OFFERS_INDEX_NAME)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(OFFERS_INDEX_NAME)
                    .table(Offers::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name(ACCOUNTS_INDEX_NAME)
                    .table(Accounts::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
