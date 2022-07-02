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
    let config = ApiConfig::load_from_s3_with_region_accounts(&shared_config, &env).await?;
    let client = Client::new(&shared_config);
    let http_client = client::get_http_client();
    let account_list = config.users.as_ref().ok_or("must have account list")?;
    let (client_map, login_failed_accounts) =
        client::get_client_map(&http_client, &config, account_list, &client).await?;

    log::info!("refresh started..");
    let failed_accounts = db::refresh_offer_cache(&client, &config, &client_map).await?;

    if env == constants::DEFAULT_AWS_REGION {
        lock::delete_all_locked_deals(&client, &config.offer_id_table_name).await?
    }

    if failed_accounts.len() > 0 || login_failed_accounts.len() > 0 {
        log::error!("failed: {:#?}", failed_accounts);
        log::error!("login failed: {:#?}", login_failed_accounts);
        return Err("accounts failed to update".into());
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
