use crate::{
    client,
    constants::mc_donalds::MCDONALDS_IMAGE_CDN,
    database::Database,
    types::{api::Offer, config::ApiConfig},
    utils,
};
use aws_sdk_s3::types::ByteStream;
use image::io::Reader as ImageReader;
use itertools::Itertools;
use mime::IMAGE_JPEG;
use std::io::Cursor;

pub async fn refresh_images(
    database: &'_ dyn Database,
    s3_client: &aws_sdk_s3::Client,
    config: &ApiConfig,
) -> Result<(), anyhow::Error> {
    let unique_offer_list = database
        .get_all_offers_as_vec()
        .await?
        .into_iter()
        .unique_by(|offer| offer.image_base_name.to_string())
        .collect();

    refresh_images_for(s3_client, config, unique_offer_list).await
}

async fn refresh_images_for(
    s3_client: &aws_sdk_s3::Client,
    config: &ApiConfig,
    offer_list: Vec<Offer>,
) -> Result<(), anyhow::Error> {
    let http_client = client::get_http_client();
    let mut new_image_count = 0;
    let mut cached_image_count = 0;

    for offer in offer_list {
        let existing = s3_client
            .head_object()
            .bucket(&config.images.bucket_name)
            .key(&offer.image_base_name)
            .send()
            .await;

        // check if exists
        if config.images.force || existing.is_err() {
            let image_url = format!("{}/{}", MCDONALDS_IMAGE_CDN, offer.image_base_name);
            let image_response = http_client.get(image_url).send().await;
            match image_response {
                Ok(image_response) => {
                    let image_bytes = image_response.bytes().await?;
                    let image = ImageReader::new(Cursor::new(image_bytes.clone()))
                        .with_guessed_format()?
                        .decode()?;
                    let webp_image_memory = webp::Encoder::from_image(&image).unwrap().encode(75.0);
                    let webp_image: Vec<u8> = webp_image_memory.iter().cloned().collect();

                    s3_client
                        .put_object()
                        .bucket(&config.images.bucket_name)
                        .key(&offer.image_base_name)
                        .content_type(IMAGE_JPEG.to_string())
                        .body(image_bytes.into())
                        .send()
                        .await?;

                    s3_client
                        .put_object()
                        .bucket(&config.images.bucket_name)
                        .key(format!(
                            "{}.webp",
                            utils::remove_ext(&offer.image_base_name)
                        ))
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
            log::debug!("{:#?} already exists in s3", offer.image_base_name)
        }
    }

    log::info!("{} new images added", new_image_count);
    log::info!("{} cached images", cached_image_count);
    Ok(())
}
