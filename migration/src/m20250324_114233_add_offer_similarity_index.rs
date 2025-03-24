use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferSimilarity {
    Table,
    OfferId,
    OtherOfferId,
    Similarity,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OfferSimilarity::Table)
                    .if_not_exists()
                    .col(big_integer(OfferSimilarity::OfferId))
                    .col(big_integer(OfferSimilarity::OtherOfferId))
                    .col(big_integer(OfferSimilarity::Similarity))
                    .primary_key(
                        Index::create()
                            .col(OfferSimilarity::OfferId)
                            .col(OfferSimilarity::OtherOfferId),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OfferSimilarity::Table).to_owned())
            .await
    }
}
