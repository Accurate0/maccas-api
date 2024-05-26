use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Extension};

#[derive(DeriveMigrationName)]
pub struct Migration;

const EXTENSION_NAME: &str = "pg_stat_statements";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let statement = Extension::create()
            .name(EXTENSION_NAME)
            .if_not_exists()
            .to_owned()
            .build_ref(&PostgresQueryBuilder);

        let db = manager.get_connection();
        db.execute_unprepared(&statement).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let statement = Extension::drop()
            .name(EXTENSION_NAME)
            .to_owned()
            .build_ref(&PostgresQueryBuilder);

        let db = manager.get_connection();
        db.execute_unprepared(&statement).await?;

        Ok(())
    }
}
