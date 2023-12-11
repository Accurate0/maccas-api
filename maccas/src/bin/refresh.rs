use anyhow::Context;
use aws_sdk_dynamodb::Client;
use foundation::aws;
use foundation::constants::{AWS_REGION, DEFAULT_AWS_REGION};
use itertools::Itertools;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::database::account::{AccountRepository, UserAccountsFilter};
use maccas::database::offer::OfferRepository;
use maccas::database::point::PointRepository;
use maccas::database::refresh::RefreshRepository;
use maccas::extensions::ApiClientExtensions;
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::images::OfferImageBaseName;
use maccas::types::sqs::{ImagesRefreshMessage, RefreshFailureMessage};
use serde_json::Value;
use std::time::Instant;

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

    let region = std::env::var(AWS_REGION)
        .context("AWS_REGION not set")
        .unwrap();

    let config = GeneralConfig::load(&shared_config).await?;
    if !config.refresh.enabled {
        log::warn!("refresh task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let offer_repository = OfferRepository::new(
        client.clone(),
        &config.database.tables,
        &config.database.indexes,
    );
    let account_repository = AccountRepository::new(client.clone(), &config.database.tables);
    let point_repository = PointRepository::new(client.clone(), &config.database.tables);
    let refresh_repository = RefreshRepository::new(client.clone(), &config.database.tables);

    let group = refresh_repository
        .increment_refresh_key(&region, config.refresh.total_groups[&region])
        .await?;

    let account_list = account_repository
        .get_user_accounts(&UserAccountsFilter {
            region: &region,
            group: &group.to_string(),
        })
        .await?;

    let (client_map, login_failed_accounts) = account_repository
        .get_api_clients(
            &config,
            &config.mcdonalds.client_id,
            &config.mcdonalds.client_secret,
            &config.mcdonalds.sensor_data,
            &account_list,
            false,
        )
        .await?;

    log::info!("refresh started..");
    let mut failed_accounts = Vec::new();
    let mut new_offers = Vec::new();

    for (account, api_client) in &client_map {
        match offer_repository
            .refresh_offer_cache(account, api_client, &config.mcdonalds.ignored_offer_ids)
            .await
        {
            Ok(mut o) => {
                new_offers.append(&mut o);
                // TODO: PointRepository
                point_repository
                    .refresh_point_cache(account, api_client)
                    .await?;
            }
            Err(e) => {
                log::error!("{}: {}", account, e);
                failed_accounts.push(account.account_name.clone());
            }
        };
    }

    log::info!(
        "refreshed {} account offer caches..",
        client_map.keys().len()
    );

    if config.refresh.clear_deal_stacks {
        log::info!("clearing all deal stacks..");
        for (account, client) in client_map {
            let account_name = account.account_name;
            log::info!("clearing deal stack for {}", account_name);
            if login_failed_accounts.contains(&account_name)
                || failed_accounts.contains(&account_name)
            {
                log::info!("skipped due to login or refresh failure");
            } else {
                client.remove_all_from_deal_stack().await;
            }
        }
    }

    if !failed_accounts.is_empty() || !login_failed_accounts.is_empty() {
        log::error!("login failed: {:#?}", login_failed_accounts);
        log::error!("refresh failed: {:#?}", failed_accounts);
    }

    if config.refresh.enable_failure_handler {
        let failed_accounts =
            [login_failed_accounts.to_owned(), failed_accounts.to_owned()].concat();

        let failed_accounts = failed_accounts.iter().unique().map(|account_name| {
            account_list
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
        let image_base_names = new_offers
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

    // only send on 1 AWS region
    if config.accounts.enabled && region == DEFAULT_AWS_REGION {
        aws::sqs::send_to_queue::<Option<String>>(&sqs_client, &config.accounts.queue_name, None)
            .await?;
    }

    // setup the last run time
    if region == DEFAULT_AWS_REGION {
        refresh_repository.set_last_refresh().await?;
    }

    log::info!(
        "completed refresh task in {} seconds",
        now.elapsed().as_secs()
    );
    Ok(())
}
