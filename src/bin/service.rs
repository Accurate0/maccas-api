use anyhow::Context;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libapi::client;
use libapi::constants::{self, mc_donalds};
use libapi::database::{Database, DynamoDatabase};
use libapi::logging;
use libapi::types::config::{GeneralConfig, UserList};
use libapi::types::images::OfferImageBaseName;
use libapi::types::sqs::{FixAccountMessage, ImagesRefreshMessage};
use libapi::types::webhook::DiscordWebhookMessage;
use serde_json::{json, Value};
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(_: LambdaEvent<Value>) -> Result<Value, anyhow::Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;
    let env = std::env::var(constants::AWS_REGION)
        .context("AWS_REGION not set")
        .unwrap();

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
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

    let embed = if config.service.refresh_offers {
        let count = database
            .increment_refresh_tracking(&env, config.service.refresh_counts[&env])
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
            let queue_url_output = sqs_client
                .get_queue_url()
                .queue_name(&config.images.queue_name)
                .send()
                .await?;

            if let Some(queue_url) = queue_url_output.queue_url() {
                let image_base_names = refresh_cache
                    .new_offers
                    .iter()
                    .map(|offer| OfferImageBaseName {
                        original: offer.original_image_base_name.clone(),
                        new: offer.image_base_name.clone(),
                    })
                    .collect();

                let queue_message = ImagesRefreshMessage { image_base_names };

                let rsp = sqs_client
                    .send_message()
                    .queue_url(queue_url)
                    .message_body(
                        serde_json::to_string(&queue_message)
                            .context("must serialize")
                            .unwrap(),
                    )
                    .send()
                    .await?;
                log::info!("added to cleanup queue: {:?}", rsp);
            } else {
                log::error!("missing queue url for {}", &config.accounts.queue_name);
            }
        }

        // send errors to the accounts queue
        if config.cleanup.enabled {
            let queue_url_output = sqs_client
                .get_queue_url()
                .queue_name(&config.accounts.queue_name)
                .send()
                .await?;

            if let Some(queue_url) = queue_url_output.queue_url() {
                for failed_account_name in &login_failed_accounts {
                    let account = account_list
                        .users
                        .iter()
                        .find(|a| a.account_name == failed_account_name.clone())
                        .unwrap()
                        .clone();

                    let queue_message = FixAccountMessage { account };

                    let rsp = sqs_client
                        .send_message()
                        .queue_url(queue_url)
                        .message_body(
                            serde_json::to_string(&queue_message)
                                .context("must serialize")
                                .unwrap(),
                        )
                        .send()
                        .await?;
                    log::info!("added to cleanup queue: {:?}", rsp);
                }
            } else {
                log::error!("missing queue url for {}", &config.accounts.queue_name);
            }
        }

        embed
            .field(EmbedFieldBuilder::new(
                "Login Failed",
                login_failed_accounts.len().to_string(),
            ))
            .field(EmbedFieldBuilder::new(
                "Refresh Failed",
                refresh_cache.failed_accounts.len().to_string(),
            ))
    } else {
        embed
    };

    if has_error && config.service.discord_refresh_error.enabled {
        let mut message = DiscordWebhookMessage::new(
            config.service.discord_refresh_error.username.clone(),
            config.service.discord_refresh_error.avatar_url.clone(),
        );

        match embed.validate() {
            Ok(embed) => {
                message.add_embed(embed.build());

                for webhook_url in &config.service.discord_refresh_error.webhooks {
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

    Ok(json!(
        {
            "isBase64Encoded": false,
            "statusCode": 204,
            "headers": {},
            "body": ""
        }
    ))
}
