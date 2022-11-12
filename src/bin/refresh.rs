use anyhow::Context;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use itertools::Itertools;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::constants::{self, mc_donalds};
use maccas::database::{Database, DynamoDatabase};
use maccas::logging;
use maccas::queue::send_to_queue;
use maccas::types::config::{GeneralConfig, UserList};
use maccas::types::images::OfferImageBaseName;
use maccas::types::sqs::ImagesRefreshMessage;
use maccas::types::webhook::DiscordWebhookMessage;
use maccas::{aws, client};
use serde_json::Value;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<Value>) -> Result<(), anyhow::Error> {
    let shared_config = aws::get_shared_config().await;

    let env = std::env::var(constants::AWS_REGION)
        .context("AWS_REGION not set")
        .unwrap();

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.refresh.enabled {
        log::warn!("refresh task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let database: Box<dyn Database> =
        Box::new(DynamoDatabase::new(client, &config.database.tables));
    let http_client = client::get_http_client();

    let mut has_error = false;
    let embed = EmbedBuilder::new()
        .color(mc_donalds::RED)
        .description("**Error**")
        .field(EmbedFieldBuilder::new("Region", &env))
        .timestamp(
            Timestamp::from_secs(Utc::now().timestamp())
                .context("must have valid time")
                .unwrap(),
        );

    let count = database
        .increment_refresh_tracking(&env, config.refresh.refresh_counts[&env])
        .await?;

    let account_list = UserList::load_from_s3(&shared_config, &env, count).await?;
    let (client_map, login_failed_accounts) = database
        .get_client_map(
            &http_client,
            &config.mcdonalds.client_id,
            &config.mcdonalds.client_secret,
            &config.mcdonalds.sensor_data,
            &account_list.users,
            false,
        )
        .await?;

    log::info!("refresh started..");
    let refresh_cache = database
        .refresh_offer_cache(&client_map, &config.mcdonalds.ignored_offer_ids)
        .await?;

    if !refresh_cache.failed_accounts.is_empty() || !login_failed_accounts.is_empty() {
        has_error = true;
        log::error!("login failed: {:#?}", login_failed_accounts);
        log::error!("refresh failed: {:#?}", refresh_cache.failed_accounts);
    }

    if config.images.enabled {
        let image_base_names = refresh_cache
            .new_offers
            .iter()
            .map(|offer| OfferImageBaseName {
                original: offer.original_image_base_name.clone(),
                new: offer.image_base_name.clone(),
            })
            .unique_by(|offer| offer.original.clone())
            .collect();

        send_to_queue(
            &sqs_client,
            &config.images.queue_name,
            ImagesRefreshMessage { image_base_names },
        )
        .await?;
    }

    if has_error {
        let embed = embed
            .field(EmbedFieldBuilder::new(
                "Login Failed",
                login_failed_accounts.len().to_string(),
            ))
            .field(EmbedFieldBuilder::new(
                "Refresh Failed",
                refresh_cache.failed_accounts.len().to_string(),
            ));

        let mut message = DiscordWebhookMessage::new(
            config.refresh.discord_error.username.clone(),
            config.refresh.discord_error.avatar_url.clone(),
        );

        match embed.validate() {
            Ok(embed) => {
                message.add_embed(embed.build());

                for webhook_url in &config.refresh.discord_error.webhooks {
                    let resp = message.send(&http_client, webhook_url).await;
                    match resp {
                        Ok(_) => {}
                        Err(e) => log::error!("{:?}", e),
                    }
                }
            }
            Err(e) => log::error!("{:?}", e),
        }
    }

    Ok(())
}
