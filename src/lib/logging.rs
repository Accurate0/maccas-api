use crate::constants::DEFAULT_TIMEZONE;
use crate::extensions::RequestExtensions;
use crate::{
    constants::{self, api_base},
    types::{jwt::JwtClaim, log::UsageLog},
};
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Tz;
use http::Method;
use http::Request;
use jwt::{Header, Token};
use reqwest_middleware::ClientWithMiddleware;
use simplelog::*;
use std::fmt::Debug;

pub fn setup_logging() {
    let term_config = ConfigBuilder::new().set_level_padding(LevelPadding::Right).build();
    TermLogger::init(LevelFilter::Info, term_config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

pub async fn log_deal_use<T: Debug>(
    http_client: &ClientWithMiddleware,
    request: &Request<T>,
    short_name: &String,
    deal_id: &String,
    api_key: &String,
    timezone: &str,
) {
    // log usage
    let auth_header = request.headers().get(http::header::AUTHORIZATION);
    if let Some(auth_header) = auth_header {
        let value = auth_header.to_str().unwrap().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();
        let correlation_id = request.get_correlation_id();
        let tz: Tz = timezone.parse().unwrap_or(DEFAULT_TIMEZONE);
        let dt = tz.from_utc_datetime(&Utc::now().naive_utc());

        let usage_log = UsageLog {
            user_id: jwt.claims().oid.to_string(),
            deal_readable: short_name.to_string(),
            deal_uuid: deal_id.to_string(),
            user_readable: jwt.claims().name.to_string(),
            message: "Deal Used",
            local_time: dt.format("%r %v %Z").to_string(),
        };

        let resp = http_client
            .request(Method::POST, format!("{}/log", api_base::LOG).as_str())
            .header(constants::LOG_SOURCE_HEADER, constants::SOURCE_NAME)
            .header(constants::CORRELATION_ID_HEADER, correlation_id)
            .header(constants::X_API_KEY_HEADER, api_key)
            .body(serde_json::to_string(&usage_log).unwrap())
            .send()
            .await;
        match resp {
            Ok(_) => {}
            Err(e) => log::error!("{:?}", e),
        }
    }
}
