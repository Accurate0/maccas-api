pub mod mc_donalds {
    pub mod default {
        pub const BASE_URL: &str = "https://ap-prod.api.mcd.com";
        pub const OFFSET: &str = "480";
        pub const STORE_ID: i64 = 951488;
        pub const FILTER: &str = "summary";
        pub const DISTANCE: &str = "10000";
        pub const LATITUDE: &str = "37.4219";
        pub const LONGITUDE: &str = "-122.084";
        pub const STORE_UNIQUE_ID_TYPE: &str = "NSN";
        pub const LOCATION_SEARCH_DISTANCE: u64 = 500;
    }
    pub const RED: u32 = 0xDA291C;
    pub const IMAGE_CDN: &str =
        "https://au-prod-us-cds-oceofferimages.s3.amazonaws.com/oce3-au-prod/offers";
}

pub mod config {
    #[cfg(debug_assertions)]
    pub const BASE_FILE: &str = "base-config-dev.json";
    #[cfg(not(debug_assertions))]
    pub const BASE_FILE: &str = "base-config.json";
    pub const REGION_ACCOUNTS_FILE: &str = "{region}/accounts-{region}-{option}.json";
    pub const CONFIG_BUCKET_NAME: &str = "maccas-application-config";
    pub const CONFIG_APIM_API_KEY_ID: &str = "MaccasApi-ApimApiKey";
    pub const CONFIG_JWT_BYPASS_ID: &str = "MaccasApi-JwtBypassKey";
    pub const CONFIG_APPLICATION_AUDIENCE_ID: &str = "MaccasApi-ApplicationAudience";
    pub const IMAGE_CDN: &str = "https://i.maccas.one";
    pub const X_JWT_BYPASS_HEADER: &str = "X-Maccas-JWT-Bypass";
    pub const DEFAULT_REFRESH_TTL_HOURS: i64 = 24;
    pub const DEFAULT_LOCK_TTL_HOURS: i64 = 12;
    pub const MAXIMUM_FAILURE_HANDLER_RETRY: i8 = 5;
    pub const MAXIMUM_CLEANUP_RETRY: i8 = 5;
    pub const MAX_PROXY_COUNT: i8 = 10;
}

pub mod db {
    pub const ACCOUNT_NAME: &str = "account_name";
    pub const LOGIN_USERNAME: &str = "login_username";
    pub const LOGIN_PASSWORD: &str = "login_password";
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
    pub const KEY: &str = "key";
    pub const VALUE: &str = "value";
    pub const OFFER_ID: &str = "offer_id";
    pub const TTL: &str = "ttl";
    pub const USER_ID: &str = "user_id";
    pub const USER_CONFIG: &str = "user_config";
    pub const USER_NAME: &str = "user_name";
    pub const DEVICE_ID: &str = "device_id";
    pub const REGION: &str = "region";
    pub const GROUP: &str = "group";
    pub const CURRENT_LIST: &str = "current_list";
    pub const TIMESTAMP: &str = "timestamp";
    pub const OFFER_NAME: &str = "offer_name";
    pub const ACTION: &str = "action";
    pub const OPERATION_ID: &str = "operation_id";
}
