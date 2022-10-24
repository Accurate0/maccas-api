use crate::types::api::{OfferDatabase, PointsResponse};
use crate::types::config::UserAccount;
use crate::types::refresh::RefreshOfferCache;
use crate::types::user::UserOptions;
use async_trait::async_trait;
use chrono::Duration;
use libmaccas::ApiClient;
use std::collections::HashMap;

#[async_trait]
pub trait Database {
    async fn get_all_offers_as_map(
        &self,
    ) -> Result<HashMap<String, Vec<OfferDatabase>>, anyhow::Error>;
    async fn get_all_offers_as_vec(&self) -> Result<Vec<OfferDatabase>, anyhow::Error>;
    async fn get_offers_for(
        &self,
        account_name: &str,
    ) -> Result<Option<Vec<OfferDatabase>>, anyhow::Error>;
    async fn set_offers_for(
        &self,
        account_name: &UserAccount,
        offer_list: &[OfferDatabase],
    ) -> Result<(), anyhow::Error>;
    async fn refresh_offer_cache(
        &self,
        client_map: &HashMap<UserAccount, ApiClient<'_>>,
        ignored_offer_ids: &[i64],
    ) -> Result<RefreshOfferCache, anyhow::Error>;
    async fn refresh_point_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
    ) -> Result<(), anyhow::Error>;
    async fn get_point_map(&self) -> Result<HashMap<String, PointsResponse>, anyhow::Error>;
    async fn get_points_by_account_hash(
        &self,
        account_hash: &str,
    ) -> Result<(UserAccount, PointsResponse), anyhow::Error>;
    async fn refresh_offer_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<OfferDatabase>, anyhow::Error>;
    async fn get_refresh_time_for_offer_cache(&self) -> Result<String, anyhow::Error>;
    async fn get_offer_by_id(
        &self,
        offer_id: &str,
    ) -> Result<(UserAccount, OfferDatabase), anyhow::Error>;
    async fn get_config_by_user_id(&self, user_id: &str) -> Result<UserOptions, anyhow::Error>;
    async fn set_config_by_user_id(
        &self,
        user_id: &str,
        user_config: &UserOptions,
        user_name: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_specific_client<'a>(
        &self,
        http_client: &'a reqwest_middleware::ClientWithMiddleware,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account: &'a UserAccount,
        force_login: bool,
    ) -> Result<ApiClient<'a>, anyhow::Error>;
    async fn get_client_map<'a>(
        &self,
        http_client: &'a reqwest_middleware::ClientWithMiddleware,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account_list: &'a [UserAccount],
        force_login: bool,
    ) -> Result<(HashMap<UserAccount, ApiClient<'a>>, Vec<String>), anyhow::Error>;
    async fn lock_deal(&self, deal_id: &str, duration: Duration) -> Result<(), anyhow::Error>;
    async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error>;
    async fn get_all_locked_deals(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn delete_all_locked_deals(&self) -> Result<(), anyhow::Error>;
    async fn get_device_id_for(&self, account_name: &str) -> Result<Option<String>, anyhow::Error>;
    async fn set_device_id_for(
        &self,
        account_name: &str,
        device_id: &str,
    ) -> Result<(), anyhow::Error>;
    async fn increment_refresh_tracking(
        &self,
        region: &str,
        max_count: i8,
    ) -> Result<i8, anyhow::Error>;
}
