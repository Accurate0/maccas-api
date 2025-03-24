use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferSimilarity {
    Table,
    OfferId,
    OtherOfferId,
}

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    PropositionId,
}

pub const OFFER_ID_FK: &str = "offer_details_proposition_id_offer_simiarity_fk";
pub const OTHER_ID_FK: &str = "offer_details_proposition_id_offer_simiarity_other_fk";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferSimilarity::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(OFFER_ID_FK)
                            .from_tbl(OfferSimilarity::Table)
                            .from_col(OfferSimilarity::OfferId)
                            .to_tbl(OfferDetails::Table)
                            .to_col(OfferDetails::PropositionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(OTHER_ID_FK)
                            .from_tbl(OfferSimilarity::Table)
                            .from_col(OfferSimilarity::OtherOfferId)
                            .to_tbl(OfferDetails::Table)
                            .to_col(OfferDetails::PropositionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferSimilarity::Table)
                    .drop_foreign_key(Alias::new(OFFER_ID_FK))
                    .drop_foreign_key(Alias::new(OTHER_ID_FK))
                    .take(),
            )
            .await
    }
}
