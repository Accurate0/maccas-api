use serde::Serialize;
use twilight_model::channel::message::Embed;

#[derive(Serialize, Debug, Default)]
pub struct DiscordWebhookMessage {
    pub(crate) content: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) avatar_url: Option<String>,
    pub(crate) tts: bool,
    pub(crate) embeds: Vec<Embed>,
}
