pub use sea_orm_migration::prelude::*;

mod m20231214_054124_create_offers_table;
mod m20231214_054356_update_offer_id;
mod m20231214_061052_add_offer_proposition_id;
mod m20231214_063221_add_accounts_table;
mod m20231214_080919_add_jobs_table;
mod m20231216_133706_offer_details_table;
mod m20231216_140842_add_fk_offer_details;
mod m20231217_051327_update_field;
mod m20231223_080257_move_fields;
mod m20231223_081714_add_created_and_updated_at;
mod m20231223_111033_add_points;
mod m20231229_104821_remove_image_basename;
mod m20231230_130059_add_refreshed_at;
mod m20231230_133157_add_stores_table;
mod m20240101_063753_rename_field_jobs;
mod m20240101_063943_add_events_table;
mod m20240118_091942_add_job_history;
mod m20240118_105246_add_offer_audit;
mod m20240127_073308_password_is_optional;
mod m20240128_092046_offer_history_table;
mod m20240130_110830_add_json_blob;
mod m20240131_150456_add_products_table;
mod m20240131_153145_add_categories_table;
mod m20240131_160651_add_products_list_to_details;
mod m20240201_090136_custom_categories_list;
mod m20240201_095751_remove_products;
mod m20240201_095903_remove_product_ids_and_add_categories;
mod m20240203_105911_update_categories;
mod m20240203_123115_update_categories_2;
mod m20240204_080304_add_account_lock;
mod m20240204_084509_add_disable_flag;
mod m20240307_122712_add_event_status;
mod m20240430_023736_add_trace_id;
mod m20240504_053110_add_refresh_failure_count;

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
            Box::new(m20231216_133706_offer_details_table::Migration),
            Box::new(m20231216_140842_add_fk_offer_details::Migration),
            Box::new(m20231217_051327_update_field::Migration),
            Box::new(m20231223_080257_move_fields::Migration),
            Box::new(m20231223_081714_add_created_and_updated_at::Migration),
            Box::new(m20231223_111033_add_points::Migration),
            Box::new(m20231229_104821_remove_image_basename::Migration),
            Box::new(m20231230_130059_add_refreshed_at::Migration),
            Box::new(m20231230_133157_add_stores_table::Migration),
            Box::new(m20240101_063753_rename_field_jobs::Migration),
            Box::new(m20240101_063943_add_events_table::Migration),
            Box::new(m20240118_091942_add_job_history::Migration),
            Box::new(m20240118_105246_add_offer_audit::Migration),
            Box::new(m20240127_073308_password_is_optional::Migration),
            Box::new(m20240128_092046_offer_history_table::Migration),
            Box::new(m20240130_110830_add_json_blob::Migration),
            Box::new(m20240131_150456_add_products_table::Migration),
            Box::new(m20240131_153145_add_categories_table::Migration),
            Box::new(m20240131_160651_add_products_list_to_details::Migration),
            Box::new(m20240201_090136_custom_categories_list::Migration),
            Box::new(m20240201_095751_remove_products::Migration),
            Box::new(m20240201_095903_remove_product_ids_and_add_categories::Migration),
            Box::new(m20240203_105911_update_categories::Migration),
            Box::new(m20240203_123115_update_categories_2::Migration),
            Box::new(m20240204_080304_add_account_lock::Migration),
            Box::new(m20240204_084509_add_disable_flag::Migration),
            Box::new(m20240307_122712_add_event_status::Migration),
            Box::new(m20240430_023736_add_trace_id::Migration),
            Box::new(m20240504_053110_add_refresh_failure_count::Migration),
        ]
    }
}
