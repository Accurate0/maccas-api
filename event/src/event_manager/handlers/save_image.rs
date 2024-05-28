use super::HandlerError;
use crate::event_manager::EventManager;
use crate::result_extension::ResultExtension;
use image::io::Reader as ImageReader;
use reqwest_middleware::ClientWithMiddleware;
use tokio::runtime::Handle;
use tracing::{Instrument, Level};

const IMAGE_BASE_URL: &str =
    "https://au-prod-us-cds-oceofferimages.s3.amazonaws.com/oce3-au-prod/offers";

pub async fn save_image(basename: String, em: EventManager) -> Result<(), HandlerError> {
    let bucket = em.get_state::<s3::Bucket>();
    let http_client = em.get_state::<ClientWithMiddleware>();

    let head_result = bucket
        .head_object(&basename)
        .instrument(tracing::span!(
            Level::INFO,
            "head object",
            basename = basename
        ))
        .await;

    if head_result.is_ok() {
        tracing::info!("image {} already exists in bucket", basename);
        return Ok(());
    }

    let url = format!("{IMAGE_BASE_URL}/{basename}");
    tracing::info!("fetching image: {}", url);

    let response = http_client.get(&url).send().await?.error_for_status()?;
    let bytes = response.bytes().await?;

    let rt = Handle::current();
    let bytes = rt
        .spawn_blocking(move || {
            let img = ImageReader::new(std::io::Cursor::new(&bytes))
                .with_guessed_format()?
                .decode()?;

            let mut bytes = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Jpeg,
            )?;

            Ok::<Vec<u8>, HandlerError>(bytes)
        })
        .instrument(tracing::span!(Level::INFO, "encode image"))
        .await
        .map_err(HandlerError::from)
        .flatten_unstable()?;

    bucket
        .put_object_with_content_type(&basename, bytes.as_ref(), "image/jpeg")
        .instrument(tracing::span!(
            Level::INFO,
            "put object",
            basename = basename
        ))
        .await?;

    Ok(())
}
