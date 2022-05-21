use async_trait::async_trait;
use config::Config;
use core::time::Duration;
use log::*;
use reqwest::header;
use reqwest::{Request, Response};
use reqwest_middleware::{Next, Result};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use serenity::prelude::*;
use simplelog::*;
use task_local_extensions::Extensions;

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
        .build();

    TermLogger::init(
        LevelFilter::Warn,
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

struct LoggingMiddleware;

#[async_trait]
impl reqwest_middleware::Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        log::warn!("Sending request {} {}", req.method(), req.url());
        let res = next.run(req, extensions).await?;
        log::warn!("Got response {}", res.status());
        Ok(res)
    }
}

pub async fn get_middleware_http_client(
    client: reqwest::Client,
) -> reqwest_middleware::ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
    reqwest_middleware::ClientBuilder::new(client)
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .with(LoggingMiddleware)
        .build()
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

    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .unwrap();

    let client = get_middleware_http_client(client).await;
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
