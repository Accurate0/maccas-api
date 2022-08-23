use crate::{
    client,
    constants::mc_donalds::MCDONALDS_IMAGE_CDN,
    database::Database,
    types::{api::Offer, config::ApiConfig},
};
use itertools::Itertools;
use mime::IMAGE_JPEG;

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
            .bucket(&config.image_bucket)
            .key(&offer.image_base_name)
            .send()
            .await;

        // check if exists
        if existing.is_err() {
            let image_url = format!("{}/{}", MCDONALDS_IMAGE_CDN, offer.image_base_name);
            let image_response = http_client.get(image_url).send().await;
            match image_response {
                Ok(image_response) => {
                    let image = image_response.bytes().await?;
                    s3_client
                        .put_object()
                        .bucket(&config.image_bucket)
                        .key(offer.image_base_name)
                        .content_type(IMAGE_JPEG.to_string())
                        .body(image.into())
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
