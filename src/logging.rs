use crate::constants::DEFAULT_TIMEZONE;
use crate::types::api::OfferDatabase;
use crate::types::config::GeneralConfig;
use crate::{
    constants::{self, api_base},
    types::log::UsageLog,
};
use anyhow::Context;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Tz;
use http::Method;
use reqwest_middleware::ClientWithMiddleware;

pub fn setup_logging() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for(
            "aws_smithy_http_tower::parse_response",
            log::LevelFilter::Warn,
        )
        .level_for(
            "aws_config::default_provider::credentials",
            log::LevelFilter::Warn,
        )
        .chain(std::io::stdout())
        .apply()
        .context("failed to set up logger")
        .unwrap();
}

pub async fn log_external(
    http_client: &ClientWithMiddleware,
    config: &GeneralConfig,
    user_id: &str,
    user_name: &str,
    offer: &OfferDatabase,
    correlation_id: &str,
) {
    let tz: Tz = config
        .api
        .log_external
        .local_time_zone
        .parse()
        .unwrap_or(DEFAULT_TIMEZONE);
    let dt = tz.from_utc_datetime(&Utc::now().naive_utc());

    let usage_log = UsageLog {
        user_id: user_id.to_string(),
        deal_readable: offer.short_name.to_string(),
        deal_uuid: offer.deal_uuid.to_string(),
        user_readable: user_name.to_string(),
        message: "Deal Used",
        local_time: dt.format("%r %v %Z").to_string(),
    };

    let resp = http_client
        .request(Method::POST, format!("{}/log", api_base::LOG).as_str())
        .header(constants::LOG_SOURCE_HEADER, constants::SOURCE_NAME)
        .header(constants::CORRELATION_ID_HEADER, correlation_id)
        .header(constants::X_API_KEY_HEADER, &config.api.api_key)
        .body(serde_json::to_string(&usage_log).unwrap())
        .send()
        .await;
    match resp {
        Ok(_) => {}
        Err(e) => log::error!("{:?}", e),
    }
}

pub fn dump_build_details() {
    log::info!("maccas-api v{}", env!("VERGEN_BUILD_SEMVER"));
    log::info!("build: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    log::info!("hash: {}", env!("VERGEN_GIT_SHA"));
    log::info!("rustc: {}", env!("VERGEN_RUSTC_SEMVER"));
}
