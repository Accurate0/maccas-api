use aws_sdk_dynamodb::Client;
use core::cache;
use core::client;
use core::config;
use core::constants;
use lambda_runtime::LambdaEvent;
use lambda_runtime::{service_fn, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(_: LambdaEvent<Value>) -> Result<Value, Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;
    let env = std::env::var(constants::MACCAS_REFRESH_REGION).unwrap();
    let config = config::load_from_s3_for_region(&shared_config, &env).await;
    let client = Client::new(&shared_config);
    let http_client = client::get_http_client();
    let client_map = client::get_client_map(&http_client, &config, &client).await?;

    cache::refresh_offer_cache(&client, &config.cache_table_name, &client_map).await?;

    Ok(json!(
        {
            "isBase64Encoded": false,
            "statusCode": 204,
            "headers": {},
            "body": ""
        }
    ))
}
