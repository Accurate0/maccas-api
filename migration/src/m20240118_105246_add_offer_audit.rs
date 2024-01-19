use sea_orm_migration::prelude::*;
use sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    PropositionId,
}

#[derive(DeriveIden)]
enum OfferAudit {
    Table,
    Id,
    TransactionId,
    PropositionId,
    UserId,
    CreatedAt,
    UpdatedAt,
    Action,
}

#[derive(DeriveIden)]
pub enum Action {
    #[sea_orm(iden = "action")]
    Type,
    Add,
    Remove,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(Action::Type)
                    .values([Action::Add, Action::Remove])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OfferAudit::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OfferAudit::Action)
                            .custom(Action::Type)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OfferAudit::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OfferAudit::TransactionId).uuid().not_null())
                    .col(
                        ColumnDef::new(OfferAudit::PropositionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OfferAudit::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(OfferAudit::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(OfferAudit::UserId).uuid().null())
                    .foreign_key(
                        ForeignKeyCreateStatement::new()
                            .name("offer_proposition_id_fk")
                            .from_tbl(OfferAudit::Table)
                            .from_col(OfferAudit::PropositionId)
                            .to_tbl(OfferDetails::Table)
                            .to_col(OfferDetails::PropositionId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(r#"
                    CREATE TRIGGER update_offer_audit_updated_at BEFORE UPDATE ON offer_audit FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
                    "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            "DROP TRIGGER IF EXISTS update_offer_audit_updated_at ON offer_audit",
        )
        .await?;

        manager
            .drop_table(Table::drop().table(OfferAudit::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(Action::Type).to_owned())
            .await
    }
}
