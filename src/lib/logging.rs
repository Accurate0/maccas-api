use crate::config::ApiConfig;
use crate::constants::mc_donalds::IMAGE_BUCKET_BASE;
use crate::constants::{mc_donalds, DEFAULT_TIMEZONE};
use crate::webhook::DiscordWebhookMessage;
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
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

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

#[allow(clippy::too_many_arguments)]
pub async fn log_deal_use(
    http_client: &ClientWithMiddleware,
    user_id: &str,
    user_name: &str,
    correlation_id: &str,
    short_name: &String,
    deal_id: &str,
    image_base_url: &String,
    config: &ApiConfig,
    store_name: &String,
) {
    // log usage
    if config
        .log
        .ignored_user_ids
        .iter()
        .any(|uid| *uid == *user_id)
    {
        log::info!("refusing to log for {}/{}", user_id, user_name);
        return;
    }

    let tz: Tz = config
        .log
        .local_time_zone
        .parse()
        .unwrap_or(DEFAULT_TIMEZONE);
    let dt = tz.from_utc_datetime(&Utc::now().naive_utc());

    let usage_log = UsageLog {
        user_id: user_id.to_string(),
        deal_readable: short_name.to_string(),
        deal_uuid: deal_id.to_string(),
        user_readable: user_name.to_string(),
        message: "Deal Used",
        local_time: dt.format("%r %v %Z").to_string(),
    };

    let resp = http_client
        .request(Method::POST, format!("{}/log", api_base::LOG).as_str())
        .header(constants::LOG_SOURCE_HEADER, constants::SOURCE_NAME)
        .header(constants::CORRELATION_ID_HEADER, correlation_id)
        .header(constants::X_API_KEY_HEADER, &config.api_key)
        .body(serde_json::to_string(&usage_log).unwrap())
        .send()
        .await;
    match resp {
        Ok(_) => {}
        Err(e) => log::error!("{:?}", e),
    }

    let mut message = DiscordWebhookMessage::new(
        config.discord.username.clone(),
        config.discord.avatar_url.clone(),
    );

    let embed = EmbedBuilder::new()
        .color(mc_donalds::RED)
        .description("**Deal Used**")
        .field(EmbedFieldBuilder::new("Name", user_name.to_string()))
        .field(EmbedFieldBuilder::new("Deal", short_name))
        .field(EmbedFieldBuilder::new("Store", store_name))
        .timestamp(
            Timestamp::from_secs(dt.timestamp())
                .context("must have valid time")
                .unwrap(),
        );

    let image = ImageSource::url(format!("{}/{}", IMAGE_BUCKET_BASE, image_base_url));
    let embed = match image {
        Ok(image) => embed.thumbnail(image),
        Err(_) => embed,
    };

    match embed.validate() {
        Ok(embed) => {
            message.add_embed(embed.build());

            for webhook_url in &config.discord.webhooks {
                let resp = message.send(http_client, webhook_url).await;
                match resp {
                    Ok(_) => {}
                    Err(e) => log::error!("{:?}", e),
                }
            }
        }
        Err(e) => log::error!("{:?}", e),
    }
}

pub fn dump_build_details() {
    log::info!("maccas-api v{}", env!("VERGEN_BUILD_SEMVER"));
    log::info!("build: {}", env!("VERGEN_BUILD_TIMESTAMP"));
    log::info!("hash: {}", env!("VERGEN_GIT_SHA"));
    log::info!("rustc: {}", env!("VERGEN_RUSTC_SEMVER"));
}
