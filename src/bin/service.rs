use anyhow::Context;
use aws_sdk_dynamodb::Client;
use chrono::Utc;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libapi::constants::{self, mc_donalds};
use libapi::database::{Database, DynamoDatabase};
use libapi::logging;
use libapi::types::config::{GeneralConfig, UserList};
use libapi::types::webhook::DiscordWebhookMessage;
use libapi::{client, images};
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
        let failed_accounts = database
            .refresh_offer_cache(&client_map, &config.mcdonalds.ignored_offer_ids)
            .await?;

        if !failed_accounts.is_empty() || !login_failed_accounts.is_empty() {
            has_error = true;
            log::error!("login failed: {:#?}", login_failed_accounts);
            log::error!("refresh failed: {:#?}", failed_accounts);
        }

        embed
            .field(EmbedFieldBuilder::new(
                "Login Failed",
                login_failed_accounts.len().to_string(),
            ))
            .field(EmbedFieldBuilder::new(
                "Refresh Failed",
                failed_accounts.len().to_string(),
            ))
    } else {
        embed
    };

    let embed = if config.service.refresh_images {
        let s3_client = aws_sdk_s3::Client::new(&shared_config);
        let image_refresh_result =
            images::refresh_images(database.as_ref(), &s3_client, &config).await;

        let image_result_message = match image_refresh_result {
            Ok(_) => "Success".to_string(),
            Err(e) => {
                has_error = true;
                e.to_string()
            }
        };

        embed.field(EmbedFieldBuilder::new("Image Status", image_result_message))
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
