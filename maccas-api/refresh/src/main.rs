use aws_sdk_dynamodb::Client;
use config::Config;
use core::cache;
use core::client;
use core::config::ApiConfig;
use lambda_runtime::LambdaEvent;
use lambda_runtime::{service_fn, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(_: LambdaEvent<Value>) -> Result<Value, Error> {
    let config = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("../../config.yml"),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present");

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let client_map = client::get_client_map(&config, &client).await?;

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
