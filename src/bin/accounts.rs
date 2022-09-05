use aws_sdk_dynamodb::Client;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libapi::client;
use libapi::constants;
use libapi::database::{Database, DynamoDatabase};
use libapi::logging;
use libapi::types::config::GeneralConfig;
use libapi::types::sqs::SqsEvent;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<SqsEvent>) -> Result<(), anyhow::Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.accounts.enabled {
        log::warn!("accounts task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let _database: Box<dyn Database> =
        Box::new(DynamoDatabase::new(client, &config.database.tables));
    let _http_client = client::get_http_client();

    Ok(())
}
