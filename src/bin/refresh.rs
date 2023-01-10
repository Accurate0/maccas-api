use anyhow::Context;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use foundation::aws;
use foundation::constants::AWS_REGION;
use foundation::types::discord::DiscordWebhookMessage;
use itertools::Itertools;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::constants::mc_donalds;
use maccas::database::types::UserAccountDatabase;
use maccas::database::{Database, DynamoDatabase};
use maccas::extensions::ApiClientExtensions;
use maccas::logging;
use maccas::types::config::{GeneralConfig, UserList};
use maccas::types::images::OfferImageBaseName;
use maccas::types::sqs::{ImagesRefreshMessage, RefreshFailureMessage};
use serde_json::Value;
use std::time::Instant;
use twilight_model::util::Timestamp;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    foundation::log::init_logger();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<Value>) -> Result<(), anyhow::Error> {
    let now = Instant::now();
    let shared_config = aws::config::get_shared_config().await;

    let env = std::env::var(AWS_REGION)
        .context("AWS_REGION not set")
        .unwrap();

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.refresh.enabled {
        log::warn!("refresh task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let database: Box<dyn Database> = Box::new(DynamoDatabase::new(
        client,
        &config.database.tables,
        &config.database.indexes,
    ));

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
    let user_list = account_list
        .users
        .iter()
        .map(UserAccountDatabase::from)
        .collect_vec();
    let (client_map, login_failed_accounts) = database
        .get_client_map(
            &config,
            &config.mcdonalds.client_id,
            &config.mcdonalds.client_secret,
            &config.mcdonalds.sensor_data,
            &user_list,
            false,
        )
        .await?;

    log::info!("refresh started..");
    let refresh_cache = database
        .refresh_offer_cache(&client_map, &config.mcdonalds.ignored_offer_ids)
        .await?;

    if config.refresh.clear_deal_stacks {
        log::info!("clearing all deal stacks..");
        for (account, client) in client_map {
            let account_name = account.account_name;
            log::info!("clearing deal stack for {}", account_name);
            if login_failed_accounts.contains(&account_name)
                || refresh_cache.failed_accounts.contains(&account_name)
            {
                log::info!("skipped due to login or refresh failure");
            } else {
                client.remove_all_from_deal_stack().await;
            }
        }
    }

    if !refresh_cache.failed_accounts.is_empty() || !login_failed_accounts.is_empty() {
        has_error = true;
        log::error!("login failed: {:#?}", login_failed_accounts);
        log::error!("refresh failed: {:#?}", refresh_cache.failed_accounts);
    }

    if config.refresh.enable_failure_handler {
        let failed_accounts = [
            login_failed_accounts.to_owned(),
            refresh_cache.failed_accounts.to_owned(),
        ]
        .concat();

        let failed_accounts = failed_accounts.iter().unique().map(|account_name| {
            account_list
                .users
                .iter()
                .find(|&account| account.account_name == *account_name)
                .unwrap()
        });

        for failed_account in failed_accounts {
            aws::sqs::send_to_queue(
                &sqs_client,
                &config.refresh.failure_queue_name,
                RefreshFailureMessage(failed_account.to_owned()),
            )
            .await?;
        }
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

        aws::sqs::send_to_queue(
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
                let http_client = foundation::http::get_default_http_client();
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

    log::info!(
        "completed refresh task in {} seconds",
        now.elapsed().as_secs()
    );
    Ok(())
}
