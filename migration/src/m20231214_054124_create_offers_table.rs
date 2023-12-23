use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Offers {
    Table,
    Id,
    OfferId,
    ValidFrom,
    ValidTo,
    Name,
    ShortName,
    Description,
    CreationDate,
    ImageBaseName,
    OriginalImageBaseName,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"CREATE OR REPLACE FUNCTION set_updated_at_column()
                RETURNS TRIGGER AS $$
                    BEGIN
                        NEW.updated_at = now();
                        RETURN NEW;
                    END;
                $$ language 'plpgsql';"#,
        )
        .await?;

        manager
            .create_table(
                Table::create()
                    .table(Offers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Offers::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Offers::OfferId).integer().not_null())
                    .col(ColumnDef::new(Offers::ValidFrom).date_time().not_null())
                    .col(ColumnDef::new(Offers::ValidTo).date_time().not_null())
                    .col(ColumnDef::new(Offers::Name).string().not_null())
                    .col(ColumnDef::new(Offers::ShortName).string().not_null())
                    .col(ColumnDef::new(Offers::Description).string().not_null())
                    .col(ColumnDef::new(Offers::CreationDate).date_time().not_null())
                    .col(ColumnDef::new(Offers::ImageBaseName).string().not_null())
                    .col(
                        ColumnDef::new(Offers::OriginalImageBaseName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Offers::CreatedAt)
                            .default(Expr::current_timestamp())
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Offers::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        db.execute_unprepared(r#"
            CREATE TRIGGER update_offers_updated_at_column BEFORE UPDATE ON offers FOR EACH ROW EXECUTE PROCEDURE set_updated_at_column();
            "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared("DROP TRIGGER IF EXISTS update_offers_updated_at_column ON offers")
            .await?;

        db.execute_unprepared("DROP FUNCTION IF EXISTS set_updated_at_column")
            .await?;

        manager
            .drop_table(Table::drop().table(Offers::Table).to_owned())
            .await
    }
}
