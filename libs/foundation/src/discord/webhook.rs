use crate::types::discord::DiscordWebhookMessage;
use twilight_model::channel::message::Embed;
use {http::header::CONTENT_TYPE, reqwest::Response, reqwest_middleware::ClientWithMiddleware};

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

    pub fn content(&mut self, content: String) -> &Self {
        self.content = Some(content);
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
