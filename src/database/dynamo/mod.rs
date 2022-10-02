use crate::types::config::Tables;

pub struct DynamoDatabase {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
    user_config_table_name: String,
    cache_table_name: String,
    cache_table_name_v2: String,
    offer_id_table_name: String,
    point_table_name: String,
    refresh_tracking: String,
}

impl DynamoDatabase {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            table_name: tables.token_cache.to_owned(),
            user_config_table_name: tables.user_config.to_owned(),
            cache_table_name: tables.offer_cache.to_owned(),
            cache_table_name_v2: tables.offer_cache_v2.to_owned(),
            offer_id_table_name: tables.offer_id.to_owned(),
            point_table_name: tables.points.to_owned(),
            refresh_tracking: tables.refresh_tracking.to_owned(),
        }
    }
}

mod db;
pub use db::*;