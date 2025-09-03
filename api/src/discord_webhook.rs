use serde::Serialize;
use twilight_model::channel::message::Embed;

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
}
