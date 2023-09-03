use crate::extensions::SecretsManagerExtensions;
use async_trait::async_trait;
use aws_sdk_secretsmanager::types::{Filter, FilterNameStringType};
use config::{AsyncSource, ConfigError, Map, ValueKind};

#[derive(Debug)]
pub struct SecretsManagerSource {
    pub(crate) secrets_client: aws_sdk_secretsmanager::Client,
    pub(crate) prefix: String,
    pub(crate) seperator: Option<String>,
    pub(crate) required: bool,
    pub(crate) remove_prefix: bool,
}

impl SecretsManagerSource {
    pub fn new(
        prefix: &str,
        secrets_client: aws_sdk_secretsmanager::Client,
    ) -> SecretsManagerSource {
        Self {
            seperator: None,
            secrets_client,
            prefix: prefix.to_owned(),
            required: true,
            remove_prefix: true,
        }
    }

    pub fn with_seperator(mut self, seperator: &str) -> Self {
        self.seperator = Some(seperator.to_owned());

        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;

        self
    }

    pub fn with_remove_prefix(mut self, remove_prefix: bool) -> Self {
        self.remove_prefix = remove_prefix;

        self
    }
}

#[async_trait]
impl AsyncSource for SecretsManagerSource {
    async fn collect(&self) -> Result<config::Map<String, config::Value>, ConfigError> {
        let filter = Filter::builder()
            .key(FilterNameStringType::Name)
            .values(&self.prefix)
            .build();

        let secrets = self
            .secrets_client
            .list_secrets()
            .filters(filter)
            .send()
            .await
            .map_err(|e| ConfigError::Foreign(Box::new(e)))?;

        // need to handle nesting...
        match secrets.secret_list() {
            Some(secrets) => {
                let prefix = match &self.seperator {
                    Some(seperator) => self
                        .prefix
                        .strip_suffix(seperator.as_str())
                        .unwrap_or(&self.prefix),
                    None => &self.prefix,
                };

                let seperator = &self.seperator.to_owned().unwrap_or("".to_owned());

                let mut map = Map::default();
                for secret in secrets {
                    let key = secret.name();
                    if let Some(key) = key {
                        log::info!("grabbing secret {}", key);
                        let value = self
                            .secrets_client
                            .get_secret(key)
                            .await
                            .map_err(|e| ConfigError::Foreign(e.into()));

                        match value {
                            Ok(value) => {
                                let key = if self.remove_prefix {
                                    key.strip_prefix(prefix).unwrap().to_owned()
                                } else {
                                    key.to_owned()
                                };

                                let key = if !seperator.is_empty() {
                                    // . is used for nesting :)
                                    // I can't find this info in docs :)
                                    key.replace(seperator, ".")
                                } else {
                                    key
                                };

                                map.insert(
                                    key.to_lowercase(),
                                    config::Value::new(
                                        Some(&"SecretsManager".to_owned()),
                                        ValueKind::String(value),
                                    ),
                                );
                            }
                            Err(e) => {
                                log::warn!("error fetching {key}: {e}");
                                if self.required {
                                    return Err(e);
                                }
                            }
                        }
                    } else {
                        continue;
                    }
                }

                Ok(map)
            }
            None => Ok(Map::default()),
        }
    }
}
