use anyhow::{bail, Context};
use aws_sdk_dynamodb::Client;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libapi::client;
use libapi::config::ApiConfig;
use libapi::constants;
use libapi::database::{Database, DynamoDatabase};
use libapi::logging;
use serde_json::{json, Value};

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
    let config = ApiConfig::load_from_s3_with_region_accounts(&shared_config, &env).await?;
    let client = Client::new(&shared_config);
    let database: Box<dyn Database> = Box::new(DynamoDatabase::new(&client, &config.tables));
    let http_client = client::get_http_client();
    let account_list = config.users.as_ref().context("must have account list")?;
    let (client_map, login_failed_accounts) = database
        .get_client_map(
            &http_client,
            &config.client_id,
            &config.client_secret,
            &config.sensor_data,
            account_list,
        )
        .await?;

    log::info!("refresh started..");
    let failed_accounts = database
        .refresh_offer_cache(&client_map, &config.ignored_offer_ids)
        .await?;

    if !failed_accounts.is_empty() || !login_failed_accounts.is_empty() {
        log::error!("failed: {:#?}", failed_accounts);
        log::error!("login failed: {:#?}", login_failed_accounts);
        bail!("accounts failed to update")
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
