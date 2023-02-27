use anyhow::Context;
use aws_sdk_s3::types::ByteStream;
use foundation::aws;
use image::io::Reader as ImageReader;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use maccas::constants::mc_donalds::IMAGE_CDN;
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::{ImagesRefreshMessage, SqsEvent};
use std::io::Cursor;

#[tokio::main]
async fn main() -> Result<(), Error> {
    foundation::log::init_logger(log::LevelFilter::Info);
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

    // batch size is currently 1
    let message = messages.first().unwrap();
    log::info!("request: {:?}", message);

    let http_client = foundation::http::get_default_http_client();
    let mut new_image_count = 0;
    let mut cached_image_count = 0;

    for offer in &message.image_base_names {
        let existing = s3_client
            .head_object()
            .bucket(&config.images.bucket_name)
            .key(&offer.new)
            .send()
            .await;

        // check if exists
        if config.images.force_refresh || existing.is_err() {
            // need the original base name to lookup against mcdonald's
            // can't just set to png, format can be jpeg
            let image_url = format!("{}/{}", IMAGE_CDN, offer.original);
            let image_response = http_client.get(image_url).send().await;
            match image_response {
                Ok(image_response) => {
                    let image_bytes = image_response.bytes().await?;
                    let image = ImageReader::new(Cursor::new(image_bytes.clone()))
                        .with_guessed_format()?
                        .decode()?;
                    let webp_image_memory = webp::Encoder::from_image(&image)
                        .unwrap()
                        .encode(config.images.webp_quality);
                    let webp_image: Vec<u8> = webp_image_memory.iter().cloned().collect();

                    if config.images.copy_originals {
                        s3_client
                            .put_object()
                            .bucket(&config.images.bucket_name)
                            .key(&offer.original)
                            .body(image_bytes.into())
                            .send()
                            .await?;
                    }

                    s3_client
                        .put_object()
                        .bucket(&config.images.bucket_name)
                        .key(&offer.new)
                        .content_type("image/webp")
                        .body(ByteStream::from(webp_image))
                        .send()
                        .await?;

                    new_image_count += 1;
                }
                Err(e) => {
                    log::error!("failed getting image for {:#?} because {}", &offer, e)
                }
            }
        } else {
            cached_image_count += 1;
            log::debug!("{:#?} already exists in s3", offer.new)
        }
    }

    log::info!("{} new images added", new_image_count);
    log::info!("{} cached images", cached_image_count);

    Ok(())
}
