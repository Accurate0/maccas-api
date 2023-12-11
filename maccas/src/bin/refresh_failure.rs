use anyhow::Context;
use foundation::aws;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::constants::config::MAXIMUM_FAILURE_HANDLER_RETRY;
use maccas::database::account::AccountRepository;
use maccas::database::offer::OfferRepository;
use maccas::database::point::PointRepository;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::{RefreshFailureMessage, SqsEvent};
use maccas::{logging, proxy};

#[tokio::main]
async fn main() -> Result<(), Error> {
    foundation::log::init_logger();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<SqsEvent>) -> Result<(), anyhow::Error> {
    let shared_config = aws::config::get_shared_config().await;
    let config = GeneralConfig::load(&shared_config).await?;
    if !config.refresh.enable_failure_handler {
        log::warn!(
            "refresh failure handler is disabled, ignoring event: {:?}",
            &event
        );
        return Ok(());
    }

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let offer_repository = OfferRepository::new(
        dynamodb_client.clone(),
        &config.database.tables,
        &config.database.indexes,
    );
    let account_repository =
        AccountRepository::new(dynamodb_client.clone(), &config.database.tables);
    let point_repository = PointRepository::new(dynamodb_client.clone(), &config.database.tables);

    let mut valid_records = event.payload.records;
    valid_records.retain(|msg| msg.body.is_some());

    let messages: Vec<RefreshFailureMessage> = valid_records
        .iter()
        .map(|msg| {
            serde_json::from_str(msg.body.as_ref().unwrap())
                .context("must deserialize")
                .unwrap()
        })
        .collect();

    // batch size is 10
    for message in messages {
        log::info!("request: {:?}", message);
        let account = message.0;
        log::info!("attempting login fix for {}", account.account_name);

        for attempt in 0..MAXIMUM_FAILURE_HANDLER_RETRY {
            log::info!("retry attempt: {}", attempt);
            let proxy = proxy::get_proxy(&config.proxy).await;
            let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
            match account_repository
                .get_api_client(
                    http_client,
                    &config.mcdonalds.client_id,
                    &config.mcdonalds.client_secret,
                    &config.mcdonalds.sensor_data,
                    &account,
                    false,
                )
                .await
            {
                Ok(api_client) => {
                    log::info!("login fixed for {}, refreshing..", account.account_name);
                    if let Err(e) = offer_repository
                        .refresh_offer_cache(
                            &account,
                            &api_client,
                            &config.mcdonalds.ignored_offer_ids,
                        )
                        .await
                    {
                        log::error!("refresh failed {}", e);
                    };

                    if let Err(e) = point_repository
                        .refresh_point_cache(&account, &api_client)
                        .await
                    {
                        log::error!("point refresh failed {}", e);
                    };

                    break;
                }
                Err(e) => {
                    log::error!("failed login for {} because {}", &account.account_name, e);
                }
            };
        }
    }

    Ok(())
}
