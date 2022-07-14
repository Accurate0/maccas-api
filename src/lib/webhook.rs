use http::header::CONTENT_TYPE;
use reqwest::Response;
use reqwest_middleware::ClientWithMiddleware;
use serde::Serialize;
use twilight_model::channel::embed::Embed;

#[derive(Serialize, Debug, Default)]
pub struct DiscordWebhookMessage {
    content: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    tts: bool,
    embeds: Vec<Embed>,
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

    pub fn add_embed(&mut self, embed: Embed) -> &Self {
        self.embeds.push(embed);
        self
    }

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
