use crate::{
    constants::mc_donalds::{self, IMAGE_CDN},
    types::{api::Offer, config::ApiConfig, webhook::DiscordWebhookMessage},
};
use anyhow::Context;
use chrono::Utc;
use http::header::CONTENT_TYPE;
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

impl DiscordWebhookMessage {
    pub async fn send(
        &self,
        http_client: &ClientWithMiddleware,
        webhook_url: &String,
    ) -> Result<Response, anyhow::Error> {
        Ok(http_client
            .post(webhook_url)
            .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
            .body(serde_json::to_string(&self)?)
            .send()
            .await?)
    }
}

pub async fn execute_discord_webhooks(
    http_client: &ClientWithMiddleware,
    config: &ApiConfig,
    user_name: &str,
    offer: &Offer,
    store_name: &str,
) {
    if !config.discord.enabled {
        return;
    }

    let mut message = DiscordWebhookMessage::new(
        config.discord.username.clone(),
        config.discord.avatar_url.clone(),
    );

    let embed = EmbedBuilder::new()
        .color(mc_donalds::RED)
        .description("**Deal Used**")
        .field(EmbedFieldBuilder::new("Name", user_name.to_string()))
        .field(EmbedFieldBuilder::new("Deal", offer.short_name.to_string()))
        .field(EmbedFieldBuilder::new("Store", store_name.to_string()))
        .timestamp(
            Timestamp::from_secs(Utc::now().timestamp())
                .context("must have valid time")
                .unwrap(),
        );

    let image = ImageSource::url(format!("{}/{}", IMAGE_CDN, offer.image_base_name));
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
