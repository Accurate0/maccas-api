use crate::types::config::{Indexes, Tables};

pub struct DynamoDatabase {
    client: aws_sdk_dynamodb::Client,
    token_cache_table_name: String,
    user_config_table_name: String,
    account_cache_table_name: String,
    offer_cache_table_name: String,
    locked_offers_table_name: String,
    point_table_name: String,
    refresh_tracking_table_name: String,
    audit_table_name: String,
    last_refresh_table_name: String,
    user_accounts: String,

    audit_user_id_index: String,
}

impl DynamoDatabase {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables, indexes: &Indexes) -> Self {
        Self {
            client,
            token_cache_table_name: tables.token_cache.to_owned(),
            user_config_table_name: tables.user_config.to_owned(),
            account_cache_table_name: tables.account_cache.to_owned(),
            offer_cache_table_name: tables.deal_cache.to_owned(),
            locked_offers_table_name: tables.locked_offers.to_owned(),
            point_table_name: tables.points.to_owned(),
            refresh_tracking_table_name: tables.refresh_tracking.to_owned(),
            audit_table_name: tables.audit.to_owned(),
            audit_user_id_index: indexes.audit_user_id_index.to_owned(),
            last_refresh_table_name: tables.last_refresh.to_owned(),
            user_accounts: tables.user_accounts.to_owned(),
        }
    }
}

mod db;
pub use db::*;
