use crate::{
    client,
    constants::mc_donalds::IMAGE_CDN,
    types::{config::GeneralConfig, images::OfferImageBaseName},
};
use aws_sdk_s3::types::ByteStream;
use image::io::Reader as ImageReader;
use std::io::Cursor;

pub async fn refresh_images(
    offer_list: &Vec<OfferImageBaseName>,
    s3_client: &aws_sdk_s3::Client,
    config: &GeneralConfig,
) -> Result<(), anyhow::Error> {
    let http_client = client::get_http_client();
    let mut new_image_count = 0;
    let mut cached_image_count = 0;

    for offer in offer_list {
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
