use aws_sdk_dynamodb::Client;
use lambda_runtime::LambdaEvent;
use lambda_runtime::{service_fn, Error};
use libapi::client;
use libapi::config::ApiConfig;
use libapi::constants;
use libapi::db;
use libapi::lock;
use libapi::logging;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(_: LambdaEvent<Value>) -> Result<Value, Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;
    let env = std::env::var(constants::AWS_REGION).unwrap();
    let config = ApiConfig::load_from_s3_for_region(&shared_config, &env).await?;
    let client = Client::new(&shared_config);
    let http_client = client::get_http_client();
    let client_map = client::get_client_map(&http_client, &config, &client).await?;

    log::info!("refresh started..");
    let failed_accounts = db::refresh_offer_cache(
        &client,
        &config.cache_table_name,
        &config.cache_table_name_v2,
        &client_map,
    )
    .await?;

    if env == constants::DEFAULT_AWS_REGION {
        lock::delete_all_locked_deals(&client, &config.offer_id_table_name).await?
    }

    Ok(json!(
        {
            "isBase64Encoded": false,
            "statusCode": if failed_accounts.len() > 0 { 400 } else { 204 },
            "headers": {},
            "body": ""
        }
    ))
}
