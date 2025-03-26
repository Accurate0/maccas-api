use super::HandlerError;
use crate::{
    discord_webhook::DiscordWebhookMessage, event_manager::EventManager, settings::Settings,
};
use anyhow::Context;
use base::{
    constants::{IMAGE_BASE_URL, IMAGE_EXT},
    feature_flag::FeatureFlagClient,
    http::get_http_client,
    jwt::generate_internal_jwt,
};
use entity::offer_details;
use recommendations::GenerateEmbeddingsFor;
use reqwest::header::CONTENT_TYPE;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{EntityTrait, QuerySelect};
use tracing::instrument;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

impl TryFrom<open_feature::StructValue> for NewOfferConfig {
    type Error = anyhow::Error;

    fn try_from(value: open_feature::StructValue) -> Result<Self, Self::Error> {
        let discord_urls = value
            .fields
            .get("discord_urls")
            .context("must have urls field")?
            .as_array()
            .context("must be array")?;

        let is_all_string = discord_urls.iter().all(|u| u.is_str());
        if !is_all_string {
            return Err(anyhow::Error::msg("all urls are not strings"));
        }

        Ok(Self {
            discord_urls: discord_urls
                .iter()
                .map(|u| u.as_str().unwrap().to_owned())
                .collect(),
        })
    }
}

struct NewOfferConfig {
    discord_urls: Vec<String>,
}

impl NewOfferConfig {
    pub fn should_notify(&self) -> bool {
        !self.discord_urls.is_empty()
    }
}

#[instrument(skip(em))]
pub async fn new_offer_found(
    offer_proposition_id: i64,
    em: EventManager,
) -> Result<(), HandlerError> {
    let feature_flag_client = em.get_state::<FeatureFlagClient>();
    let settings = em.get_state::<Settings>();

    let should_send_event = feature_flag_client
        .is_feature_enabled("maccas-event-new-offer-notification", false)
        .await;

    let config = feature_flag_client
        .get_dynamic_config::<NewOfferConfig>("maccas-event-new-offer-config")
        .await;

    let should_generate_embedding = feature_flag_client
        .is_feature_enabled("maccas-api-enable-recommendations", false)
        .await;

    if should_generate_embedding {
        let http_client = get_http_client()?;
        let token = generate_internal_jwt(
            settings.auth_secret.as_ref(),
            "Maccas Event",
            "Maccas Recommendations",
        )?;

        let request_url = format!(
            "{}/{}",
            settings.recommendations_api_base,
            GenerateEmbeddingsFor::path(offer_proposition_id)
        );

        let request = http_client.post(&request_url).bearer_auth(token);

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    tracing::info!("started task");
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }
    }

    let should_notify = config.as_ref().is_some_and(|c| c.should_notify());
    if !should_send_event || config.is_none() || !should_notify {
        tracing::warn!("notification disabled or no urls configured");
        return Ok(());
    }

    let config = config.unwrap();
    let db = em.db();
    let details = offer_details::Entity::find_by_id(offer_proposition_id)
        .limit(1)
        .one(db)
        .await?;

    if details.is_none() {
        tracing::warn!("details not found for {offer_proposition_id}");
    }

    let details = details.unwrap();

    let embed = EmbedBuilder::new()
        .color(0xDA291C)
        .title("New Deal")
        .field(EmbedFieldBuilder::new("Name", details.short_name))
        .timestamp(
            Timestamp::from_secs(details.created_at.and_utc().timestamp())
                .context("must have valid time")
                .unwrap(),
        );

    let image = ImageSource::url(format!(
        "{IMAGE_BASE_URL}/{}.{IMAGE_EXT}",
        details.image_base_name
    ));

    let embed = match image {
        Ok(image) => embed.thumbnail(image),
        Err(_) => embed,
    }
    .build();

    let mut webhook_message =
        DiscordWebhookMessage::new("Maccas".to_owned(), format!("{IMAGE_BASE_URL}/og.png"));
    let webhook_message = webhook_message.add_embed(embed);

    let http_client = em.get_state::<ClientWithMiddleware>();

    for discord_url in config.discord_urls {
        http_client
            .post(discord_url)
            .header(CONTENT_TYPE, "application/json")
            .json(webhook_message)
            .send()
            .await?;
    }

    Ok(())
}
