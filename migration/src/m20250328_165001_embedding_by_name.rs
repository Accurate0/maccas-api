use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferEmbeddings {
    Table,
    Name,
    Embeddings,
    PropositionId,
}

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    PropositionId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                TableDropStatement::new()
                    .table(OfferEmbeddings::Table)
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OfferEmbeddings::Table)
                    .col(string_uniq(OfferEmbeddings::Name).primary_key())
                    .col(
                        ColumnDef::new(OfferEmbeddings::Embeddings)
                            .vector(None)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OfferEmbeddings::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OfferEmbeddings::Table)
                    .col(big_integer(OfferEmbeddings::PropositionId).primary_key())
                    .col(
                        ColumnDef::new(OfferEmbeddings::Embeddings)
                            .vector(None)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("embedding_to_details_fk")
                            .from_tbl(OfferEmbeddings::Table)
                            .from_col(OfferEmbeddings::PropositionId)
                            .to_tbl(OfferDetails::Table)
                            .to_col(OfferDetails::PropositionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }
}
