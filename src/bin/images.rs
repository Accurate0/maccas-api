use anyhow::Context;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libapi::constants;
use libapi::images;
use libapi::logging;
use libapi::types::config::GeneralConfig;
use libapi::types::sqs::{ImagesRefreshMessage, SqsEvent};

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
    if !config.images.enabled {
        log::warn!("images task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let s3_client = aws_sdk_s3::Client::new(&shared_config);

    let mut valid_records = event.payload.records;
    valid_records.retain(|msg| msg.body.is_some());

    let messages: Vec<ImagesRefreshMessage> = valid_records
        .iter()
        .map(|msg| {
            serde_json::from_str(msg.body.as_ref().unwrap())
                .context("must deserialize")
                .unwrap()
        })
        .collect();

    // batch size is currently 1 so this loop is redundant..
    for message in messages {
        log::info!("request: {:?}", message);

        images::refresh_images(&message.image_base_names, &s3_client, &config).await?;
    }

    Ok(())
}
