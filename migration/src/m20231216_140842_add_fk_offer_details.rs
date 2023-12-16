use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    PropositionId,
}

#[derive(DeriveIden)]
enum Offers {
    Table,
    OfferPropositionId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Offers::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("offer_proposition_id")
                            .from_tbl(Offers::Table)
                            .from_col(Offers::OfferPropositionId)
                            .to_tbl(OfferDetails::Table)
                            .to_col(OfferDetails::PropositionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKeyDropStatement::new()
                    .table(Offers::Table)
                    .name("offer_proposition_id")
                    .to_owned(),
            )
            .await
    }
}
