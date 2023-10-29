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
    // TODO: config file
    #[cfg(debug_assertions)]
    pub const BASE_FILE: &str = "base-config-dev.json";
    #[cfg(not(debug_assertions))]
    pub const BASE_FILE: &str = "base-config.json";
    pub const CONFIG_BUCKET_NAME: &str = "maccas-application-config";
    pub const CONFIG_BASE_URL: &str = "https://api.maccas.one/v1";
    pub const CONFIG_PLACES_API_KEY_ID: &str = "MaccasApi-PlacesApiKey";
    pub const CONFIG_SECRET_KEY_ID: &str = "MaccasApi-SecretKey";
    pub const IMAGE_CDN: &str = "https://i.maccas.one";
    pub const WEBSITE_BASE_URL: &str = "https://maccas.one";
    pub const DEFAULT_REFRESH_TTL_HOURS: i64 = 24;
    pub const DEFAULT_LOCK_TTL_HOURS: i64 = 12;
    pub const MAXIMUM_FAILURE_HANDLER_RETRY: i8 = 5;
    pub const MAXIMUM_CLEANUP_RETRY: i8 = 5;
    pub const MAX_PROXY_COUNT: i8 = 10;
    pub const TOKEN_VALID_TIME: i64 = 86400;
    pub const TOKEN_ACCESS_ISS: &str = "Maccas API";
}

pub mod db {
    pub const USERNAME: &str = "username";
    pub const ACCOUNT_NAME: &str = "account_name";
    pub const LOGIN_USERNAME: &str = "login_username";
    pub const LOGIN_PASSWORD: &str = "login_password";
    pub const ACCOUNT_HASH: &str = "account_hash";
    pub const ACCOUNT_INFO: &str = "account_info";
    pub const ACCESS_TOKEN: &str = "access_token";
    pub const TOKEN: &str = "token";
    pub const REFRESH_TOKEN: &str = "refresh_token";
    pub const LAST_REFRESH: &str = "last_refresh";
    pub const POINT_INFO: &str = "point_info";
    pub const OFFER_LIST: &str = "offer_list";
    pub const OFFER: &str = "offer";
    pub const OFFER_PROPOSITION_ID: &str = "offer_proposition_id";
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
    pub const ACTOR: &str = "actor";
    pub const DEVICE_ID: &str = "device_id";
    pub const REGION: &str = "region";
    pub const GROUP: &str = "group";
    pub const CURRENT_LIST: &str = "current_list";
    pub const TIMESTAMP: &str = "timestamp";
    pub const OFFER_NAME: &str = "offer_name";
    pub const ACTION: &str = "action";
    pub const OPERATION_ID: &str = "operation_id";
    pub const PASSWORD_HASH: &str = "password_hash";
    pub const SALT: &str = "salt";
    pub const IS_IMPORTED: &str = "is_imported";
    pub const REGISTRATION_TOKEN: &str = "registration_token";
    pub const ROLE: &str = "role";
    pub const ONE_TIME_TOKEN: &str = "one_time_token";
    pub const USES: &str = "uses";
}
