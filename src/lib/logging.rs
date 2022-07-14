use crate::config::ApiConfig;
use crate::constants::{mc_donalds, DEFAULT_TIMEZONE};
use crate::extensions::RequestExtensions;
use crate::webhook::{execute_discord_webhook, DiscordWebhookMessage};
use crate::{
    constants::{self, api_base},
    types::{jwt::JwtClaim, log::UsageLog},
};
use anyhow::Context;
use chrono::TimeZone;
use chrono::Utc;
use chrono_tz::Tz;
use http::Method;
use http::Request;
use jwt::{Header, Token};
use reqwest_middleware::ClientWithMiddleware;
use simplelog::*;
use std::fmt::Debug;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

pub fn setup_logging() {
    let term_config = ConfigBuilder::new().set_level_padding(LevelPadding::Right).build();
    TermLogger::init(LevelFilter::Info, term_config, TerminalMode::Mixed, ColorChoice::Auto).unwrap();
}

pub async fn log_deal_use<T: Debug>(
    http_client: &ClientWithMiddleware,
    request: &Request<T>,
    ignored_user_ids: &[String],
    short_name: &String,
    deal_id: &String,
    image_base_url: &String,
    config: &ApiConfig,
) {
    // log usage
    let auth_header = request.headers().get(http::header::AUTHORIZATION);
    if let Some(auth_header) = auth_header {
        let value = auth_header.to_str().unwrap().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();

        if ignored_user_ids.iter().any(|user_id| *user_id == jwt.claims().oid) {
            log::info!("refusing to log for {}/{}", jwt.claims().oid, jwt.claims().name);
            return;
        }

        let correlation_id = request.get_correlation_id();
        let tz: Tz = config.local_time_zone.parse().unwrap_or(DEFAULT_TIMEZONE);
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
            .header(constants::X_API_KEY_HEADER, &config.api_key)
            .body(serde_json::to_string(&usage_log).unwrap())
            .send()
            .await;
        match resp {
            Ok(_) => {}
            Err(e) => log::error!("{:?}", e),
        }

        let mut message =
            DiscordWebhookMessage::new(config.discord.username.clone(), config.discord.avatar_url.clone());

        let embed = EmbedBuilder::new()
            .color(mc_donalds::RED)
            .description("**Deal Used**")
            .field(EmbedFieldBuilder::new("Name", jwt.claims().name.to_string()))
            .field(EmbedFieldBuilder::new("Deal", short_name))
            .timestamp(
                Timestamp::from_secs(dt.timestamp())
                    .context("must have valid time")
                    .unwrap(),
            );

        let image = ImageSource::url(format!("{}/{}", mc_donalds::IMAGE_BUCKET, image_base_url));
        let embed = match image {
            Ok(image) => embed.thumbnail(image),
            Err(_) => embed,
        };

        match embed.validate() {
            Ok(embed) => {
                message.embeds.push(embed.build());

                for webhook_url in &config.discord.webhooks {
                    let resp = execute_discord_webhook(http_client, webhook_url, &message).await;
                    match resp {
                        Ok(_) => {}
                        Err(e) => log::error!("{:?}", e),
                    }
                }
            }
            Err(e) => log::error!("{:?}", e),
        }
    } else {
        log::info!("request with no auth header, skipping deal log");
    }
}
