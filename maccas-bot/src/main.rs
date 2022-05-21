use config::Config;
use log::*;
use maccas_core::client;
use reqwest::header;
use serenity::prelude::*;
use simplelog::*;

mod api;
mod code;
mod constants;
mod deals;
mod event_handler;
mod location;
mod refresh;
mod remove;

struct Bot {
    api_client: api::Api,
}

fn setup_logging() {
    let term_config = ConfigBuilder::new()
        .set_level_padding(LevelPadding::Right)
        .add_filter_ignore_str("tracing::span")
        .build();

    TermLogger::init(
        LevelFilter::Info,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

#[derive(serde::Deserialize, std::fmt::Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotConfig {
    pub api_key: String,
    pub base_url: String,
    pub discord_token: String,
}

#[tokio::main]
async fn main() {
    setup_logging();

    let config = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("../config.yml"),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap()
        .try_deserialize::<BotConfig>()
        .expect("valid configuration present");

    let mut api_key_header = header::HeaderValue::from_str(config.api_key.as_str()).unwrap();
    api_key_header.set_sensitive(true);

    let mut headers = header::HeaderMap::new();
    headers.insert("X-Api-Key", api_key_header);

    let client = client::get_http_client_with_headers(headers);
    let base_url = reqwest::Url::parse(&config.base_url.as_str()).unwrap();
    let api_client = api::Api { base_url, client };
    let bot = Bot { api_client };

    let mut discord_client = Client::builder(
        config.discord_token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(bot)
    .await
    .expect("Error creating client");

    if let Err(why) = discord_client.start().await {
        println!("Client error: {:?}", why);
    }
}
