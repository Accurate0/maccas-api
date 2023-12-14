pub use sea_orm_migration::prelude::*;

mod m20231214_054124_create_offers_table;
mod m20231214_054356_update_offer_id;
mod m20231214_061052_add_offer_proposition_id;
mod m20231214_063221_add_accounts_table;
mod m20231214_080919_add_jobs_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231214_054124_create_offers_table::Migration),
            Box::new(m20231214_054356_update_offer_id::Migration),
            Box::new(m20231214_061052_add_offer_proposition_id::Migration),
            Box::new(m20231214_063221_add_accounts_table::Migration),
            Box::new(m20231214_080919_add_jobs_table::Migration),
        ]
    }
}
