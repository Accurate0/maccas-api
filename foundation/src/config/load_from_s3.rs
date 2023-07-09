pub async fn load_config_from_s3<T>(
    client: &aws_sdk_s3::Client,
    bucket: impl ToString,
    file: impl ToString,
    format: config::FileFormat,
) -> Result<T, anyhow::Error>
where
    T: serde::de::DeserializeOwned,
{
    use config::Config;
    let file = file.to_string();
    let bucket = bucket.to_string();

    log::info!(
        "loading configuration from {}/{} for type: {}",
        &bucket,
        &file,
        std::any::type_name::<T>()
    );

    let resp = client
        .get_object()
        .bucket(bucket)
        .key(file)
        .send()
        .await?
        .body
        .collect()
        .await?;

    let config = Config::builder().add_source(config::File::from_str(
        std::str::from_utf8(&resp.into_bytes())?,
        format,
    ));

    Ok(config.build()?.try_deserialize::<T>()?)
}
