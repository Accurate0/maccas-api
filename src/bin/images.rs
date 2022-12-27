use anyhow::Context;
use foundation::aws;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::images;
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::{ImagesRefreshMessage, SqsEvent};

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
