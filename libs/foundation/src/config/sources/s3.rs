use async_trait::async_trait;
use aws_sdk_s3::operation::get_object::GetObjectError;
use config::{AsyncSource, ConfigError, Map, Source};

#[derive(Debug)]
pub struct S3Source {
    pub(crate) s3_client: aws_sdk_s3::Client,
    pub(crate) bucket: String,
    pub(crate) file: String,
    pub(crate) required: bool,
    pub(crate) format: config::FileFormat,
}

impl S3Source {
    pub fn new(
        bucket: &str,
        file: &str,
        format: config::FileFormat,
        s3_client: aws_sdk_s3::Client,
    ) -> S3Source {
        Self {
            bucket: bucket.to_owned(),
            s3_client,
            file: file.to_owned(),
            format,
            required: true,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;

        self
    }
}

#[async_trait]
impl AsyncSource for S3Source {
    async fn collect(&self) -> Result<config::Map<String, config::Value>, ConfigError> {
        let file = &self.file;
        let bucket = &self.bucket;

        log::info!("loading configuration from {}/{}", &bucket, &file);

        let resp = self
            .s3_client
            .get_object()
            .bucket(bucket)
            .key(file)
            .send()
            .await
            .map_err(|e| e.into_service_error());

        let resp = match resp {
            Ok(resp) => resp
                .body
                .collect()
                .await
                .map_err(|e| ConfigError::Foreign(Box::new(e)))?,

            Err(e) => match e {
                GetObjectError::NoSuchKey(e) => {
                    if self.required {
                        return Err(ConfigError::Foreign(Box::new(e)));
                    } else {
                        return Ok(Map::default());
                    }
                }
                e => {
                    log::error!("failed fetching {} because {}", file, e);
                    return Err(ConfigError::Foreign(Box::new(e)));
                }
            },
        };

        config::File::from_str(
            std::str::from_utf8(&resp.into_bytes())
                .map_err(|e| ConfigError::Foreign(Box::new(e)))?,
            self.format,
        )
        .collect()
    }
}
