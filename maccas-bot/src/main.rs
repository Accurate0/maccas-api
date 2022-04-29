use config::Config;
use log::*;
use reqwest::header;
use serenity::prelude::*;
use simplelog::*;

mod api;
mod code;
mod constants;
mod deals;
mod event_handler;
mod refresh;
mod remove;

struct Bot {
    api_client: api::Api,
}

fn setup_logging() {
    let term_config = ConfigBuilder::new()
        .set_level_padding(LevelPadding::Right)
        .build();

    TermLogger::init(
        LevelFilter::Info,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct BotConfig {
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
    // APIM randomly complains about this... we literally don't have content length ever
    headers.insert("Content-Length", header::HeaderValue::from(0 as i32));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let base_url = reqwest::Url::parse(&config.base_url.as_str()).unwrap();
    let api_client = api::Api { base_url, client };
    let bot = Bot { api_client };

    let mut client = Client::builder(
        config.discord_token,
        GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(bot)
    .await
    .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
