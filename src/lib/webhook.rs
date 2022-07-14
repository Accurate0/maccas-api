use http::header::CONTENT_TYPE;
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use twilight_model::channel::embed::Embed;

#[derive(Serialize, Debug, Default)]
pub struct DiscordWebhookMessage {
    pub content: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
    pub tts: bool,
    pub embeds: Vec<Embed>,
}

impl DiscordWebhookMessage {
    pub fn new(username: String, avatar_url: String) -> Self {
        Self {
            content: None,
            username: Some(username),
            avatar_url: Some(avatar_url),
            tts: false,
            embeds: vec![],
        }
    }
}

pub async fn execute_discord_webhook(
    http_client: &ClientWithMiddleware,
    webhook_url: &String,
    message: &DiscordWebhookMessage,
) -> Result<Response, anyhow::Error> {
    let response = http_client
        .post(webhook_url)
        .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
        .body(serde_json::to_string(message)?)
        .send()
        .await?;
    Ok(response)
}
