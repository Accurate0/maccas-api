use crate::{
    constants::{config::IMAGE_CDN, mc_donalds},
    database::types::OfferDatabase,
    types::config::GeneralConfig,
};
use anyhow::Context;
use chrono::Utc;
use foundation::types::discord::DiscordWebhookMessage;
use reqwest_middleware::ClientWithMiddleware;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

pub async fn execute_discord_webhooks(
    http_client: &ClientWithMiddleware,
    config: &GeneralConfig,
    user_name: &str,
    offer: &OfferDatabase,
    store_name: &str,
) {
    let mut message = DiscordWebhookMessage::new(
        config.api.discord_deal_use.username.clone(),
        config.api.discord_deal_use.avatar_url.clone(),
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

            for webhook_url in &config.api.discord_deal_use.webhooks {
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
