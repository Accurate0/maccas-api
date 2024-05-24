use super::HandlerError;
use crate::{event_manager::EventManager, settings::Settings};
use base::http::get_simple_http_client;
use image::io::Reader as ImageReader;
use s3::{creds::Credentials, Bucket, Region};

const IMAGE_BASE_URL: &str =
    "https://au-prod-us-cds-oceofferimages.s3.amazonaws.com/oce3-au-prod/offers";

const BUCKET_NAME: &str = "maccas-api-images";

pub async fn save_image(basename: String, em: EventManager) -> Result<(), HandlerError> {
    let settings = em.get_state::<Settings>();
    let credentials = Credentials::new(
        Some(&settings.images_bucket.access_key_id),
        Some(&settings.images_bucket.access_secret_key),
        None,
        None,
        None,
    )?;

    let bucket = Bucket::new(
        BUCKET_NAME,
        Region::Custom {
            region: "apac".to_owned(),
            endpoint: settings.images_bucket.endpoint.clone(),
        },
        credentials,
    )?;

    // FIXME: clone addict
    let head_result = bucket.head_object(basename.clone()).await;
    if head_result.is_ok() {
        tracing::info!("image {} already exists in bucket", basename);
        return Ok(());
    }

    let http_client = get_simple_http_client()?;

    let url = format!("{IMAGE_BASE_URL}/{basename}");
    tracing::info!("fetching image: {}", url);

    let response = http_client.get(&url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let img = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()?
        .decode()?;

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Jpeg,
    )?;

    bucket
        .put_object_with_content_type(basename, bytes.as_ref(), "image/jpeg")
        .await?;

    Ok(())
}
