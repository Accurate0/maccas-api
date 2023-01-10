use anyhow::Context;
use aws_sdk_dynamodb::Client;
use foundation::aws;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::constants::config::MAXIMUM_CLEANUP_RETRY;
use maccas::constants::mc_donalds::default;
use maccas::database::types::AuditActionType;
use maccas::database::{Database, DynamoDatabase};
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::{CleanupMessage, SqsEvent};
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

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.cleanup.enabled {
        log::warn!("cleanup task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let database: Box<dyn Database> = Box::new(DynamoDatabase::new(
        client,
        &config.database.tables,
        &config.database.indexes,
    ));

    let locked_deals = database.get_all_locked_deals().await?;
    let mut valid_records = event.payload.records;
    valid_records.retain(|msg| msg.body.is_some());

    let messages: Vec<CleanupMessage> = valid_records
        .iter()
        .map(|msg| {
            serde_json::from_str(msg.body.as_ref().unwrap())
                .context("must deserialize")
                .unwrap()
        })
        .collect();

    // batch size is currently 1
    let message = messages.first().unwrap();
    log::info!("request: {:?}", message);
    if !locked_deals.contains(&message.deal_uuid) {
        log::warn!("skipping processing of deal - {}", &message.deal_uuid);
        return Ok(());
    }

    let user_id = message.user_id.clone();
    let (account, offer) = database.get_offer_by_id(&message.deal_uuid).await?;

    for _ in 0..MAXIMUM_CLEANUP_RETRY {
        let proxy = proxy::get_proxy(&config);
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);

        match database
            .get_specific_client(
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
                log::info!("offer: {:?}", offer);

                let deal_stack = api_client
                    .get_offers_dealstack(default::OFFSET, &message.store_id)
                    .await?
                    .body
                    .response;

                if deal_stack.is_none() {
                    log::info!("no deal stack for {}", account);
                    continue;
                }

                if deal_stack.as_ref().unwrap().deal_stack.is_none() {
                    log::info!("no deal stack for {}", account);
                    continue;
                }

                let deal_stack = deal_stack.unwrap().deal_stack.unwrap();
                let matched_item = deal_stack.iter().find(|item| {
                    item.offer_id == offer.offer_id
                        && item.offer_proposition_id.parse::<i64>()
                            == Ok(offer.offer_proposition_id)
                });

                match matched_item {
                    Some(item) => {
                        let response = api_client
                            .remove_from_offers_dealstack(
                                &item.offer_id,
                                &item.offer_proposition_id,
                                default::OFFSET,
                                &message.store_id,
                            )
                            .await?;

                        database
                            .add_to_audit(AuditActionType::Remove, user_id, None, &offer)
                            .await;

                        log::info!("removed from dealstack - {}", response.status);

                        database.unlock_deal(&message.deal_uuid).await?;
                        log::info!("unlocked deal - {}", &message.deal_uuid);
                    }
                    None => {
                        log::info!("no matched item for {:?} - {}", offer, account);
                    }
                }

                break;
            }
            Err(e) => {
                log::info!("cleanup failed for {} because {}", message.deal_uuid, e);
            }
        };
    }

    Ok(())
}
