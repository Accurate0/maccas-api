mod code;
mod deals;
mod deals_add_remove;
mod deals_lock;
mod fallback;
mod last_refresh;
mod locations;
mod locations_search;
mod user_config;

pub use code::Code;
pub use deals::Deals;
pub use deals_add_remove::DealsAddRemove;
pub use deals_lock::DealsLock;
pub use fallback::Fallback;
pub use last_refresh::LastRefresh;
pub use locations::Locations;
pub use locations_search::LocationsSearch;
pub use user_config::UserConfig;

pub struct Context {
    pub api_config: crate::config::ApiConfig,
    pub dynamodb_client: aws_sdk_dynamodb::Client,
}
