use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum OfferDetails {
    Table,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .add_column(
                        ColumnDef::new(OfferDetails::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .add_column(
                        ColumnDef::new(OfferDetails::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE OR REPLACE FUNCTION update_offer_details_updated_at_column()
                    RETURNS TRIGGER AS $$
                        BEGIN
                            NEW.updated_at = now();
                            RETURN NEW;
                        END;
                    $$ language 'plpgsql';"#,
        )
        .await?;

        db.execute_unprepared(r#"
                CREATE TRIGGER update_offer_details_updated_at_column BEFORE UPDATE ON offer_details FOR EACH ROW EXECUTE PROCEDURE update_offer_details_updated_at_column();
                "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            "DROP TRIGGER IF EXISTS update_offer_details_updated_at_column ON offer_details",
        )
        .await?;

        db.execute_unprepared("DROP FUNCTION IF EXISTS update_offer_details_updated_at_column")
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(OfferDetails::Table)
                    .drop_column(OfferDetails::CreatedAt)
                    .drop_column(OfferDetails::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}
