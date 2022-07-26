pub const CONFIG_BUCKET_NAME: &str = "maccas-application-config";
pub const DEFAULT_AWS_REGION: &str = "ap-southeast-2";
pub const AWS_REGION: &str = "AWS_REGION";
pub const AWS_LAMBDA_FUNCTION_NAME: &str = "AWS_LAMBDA_FUNCTION_NAME";
pub const X_API_KEY_HEADER: &str = "X-Api-Key";
pub const MACCAS_WEB_API_PREFIX: &str = "MACCAS_WEB_";
pub const LOCATION_SEARCH_DISTANCE: u64 = 500;
pub const CORRELATION_ID_HEADER: &str = "traceparent";
pub const X_LOG_USER_ID: &str = "x-log-user-id";
pub const X_LOG_USER_NAME: &str = "x-log-user-name";
pub const LOG_SOURCE_HEADER: &str = "X-Source";
pub const SOURCE_NAME: &str = "MaccasWeb";
pub const DEFAULT_TIMEZONE: chrono_tz::Tz = chrono_tz::Australia::Perth;

pub mod mc_donalds {
    pub mod default {
        pub const BASE_URL: &str = "https://ap-prod.api.mcd.com";
        pub const OFFSET: &str = "480";
        pub const STORE_ID: i64 = 951488;
        pub const FILTER: &str = "summary";
        pub const DISTANCE: &str = "10000";
        pub const LATITUDE: &str = "37.4219";
        pub const LONGITUDE: &str = "-122.084";
    }
    pub const RED: u32 = 0xDA291C;
    pub const IMAGE_BUCKET: &str =
        "https://au-prod-us-cds-oceofferimages.s3.amazonaws.com/oce3-au-prod/offers";
}

pub mod api_base {
    pub const KVP: &str = "https://api.anurag.sh/kvp/v1";
    pub const PLACES: &str = "https://api.anurag.sh/places/v1";
    pub const LOG: &str = "https://api.anurag.sh/log/v1";
    pub const THIS: &str = "https://api.anurag.sh/maccas/v1";
}

pub mod config {
    pub const BASE_FILE: &str = "base-config.json";
    pub const ALL_ACCOUNTS_FILE: &str = "accounts.json";
    pub const REGION_ACCOUNTS_FILE: &str = "accounts-{region}.json";
}

pub mod db {
    pub const ACCOUNT_NAME: &str = "account_name";
    pub const ACCOUNT_HASH: &str = "account_hash";
    pub const ACCOUNT_INFO: &str = "account_info";
    pub const ACCESS_TOKEN: &str = "access_token";
    pub const REFRESH_TOKEN: &str = "refresh_token";
    pub const LAST_REFRESH: &str = "last_refresh";
    pub const POINT_INFO: &str = "point_info";
    pub const OFFER_LIST: &str = "offer_list";
    pub const OFFER: &str = "offer";
    pub const DATA_TYPE: &str = "data_type";
    pub const LOCKED_DEALS: &str = "locked_deals";
    pub const DEAL_UUID: &str = "deal_uuid";
    pub const VALUE: &str = "value";
    pub const OFFER_ID: &str = "offer_id";
    pub const TTL: &str = "ttl";
    pub const USER_ID: &str = "user_id";
    pub const USER_CONFIG: &str = "user_config";
    pub const USER_NAME: &str = "user_name";
    pub const DEVICE_ID: &str = "device_id";
}
