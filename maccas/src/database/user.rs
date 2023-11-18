use super::types::{RegistrationTokenMetadata, User, UserOptionsDatabase};
use crate::{
    constants::db::{
        ACCESS_TOKEN, IS_IMPORTED, ONE_TIME_TOKEN, PASSWORD_HASH, REFRESH_TOKEN,
        REGISTRATION_TOKEN, ROLE, SALT, TIMESTAMP, TOKEN, TTL, USERNAME, USER_CONFIG, USER_ID,
        USER_NAME, USES,
    },
    extensions::StringExtensions,
    types::{config::Tables, role::UserRole},
};
use anyhow::{bail, Context};
use aws_sdk_dynamodb::{
    primitives::Blob,
    types::{AttributeValue, AttributeValueUpdate},
};
use chrono::{DateTime, Duration, Utc};
use std::time::SystemTime;

pub struct UserRepository {
    client: aws_sdk_dynamodb::Client,
    user_tokens: String,
    users: String,
    registration_tokens: String,
    user_config: String,
}

impl UserRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            user_tokens: tables.user_tokens.clone(),
            users: tables.users.clone(),
            registration_tokens: tables.registration_tokens.clone(),
            user_config: tables.user_config.clone(),
        }
    }

    pub async fn set_user_tokens(
        &self,
        username: &str,
        auth_token: &str,
        refresh_token: Vec<String>,
        ttl: Duration,
    ) -> Result<(), anyhow::Error> {
        let utc: DateTime<Utc> = Utc::now().checked_add_signed(ttl).unwrap();

        self.client
            .put_item()
            .table_name(&self.user_tokens)
            .item(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .item(ACCESS_TOKEN, AttributeValue::S(auth_token.to_owned()))
            .item(REFRESH_TOKEN, AttributeValue::Ss(refresh_token))
            .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_user_tokens(
        &self,
        username: String,
    ) -> Result<(String, Vec<String>), anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.user_tokens)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let access_token = item
            .get(ACCESS_TOKEN)
            .context("must have access token")?
            .as_s()
            .cloned()
            .unwrap();

        let refresh_token = item
            .get(REFRESH_TOKEN)
            .context("must have refresh token")?
            .as_ss()
            .cloned()
            .unwrap_or_else(|_| vec![item.get(REFRESH_TOKEN).unwrap().as_s().cloned().unwrap()]);

        Ok((access_token, refresh_token))
    }

    pub async fn get_user_id(&self, username: String) -> Result<String, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let user_id = item
            .get(USER_ID)
            .context("must have user id")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(user_id)
    }

    pub async fn get_user_role(&self, username: String) -> Result<UserRole, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let role = item
            .get(ROLE)
            .context("must have password")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(serde_json::from_str::<UserRole>(&role)?)
    }

    pub async fn set_user_role(
        &self,
        username: String,
        role: UserRole,
    ) -> Result<(), anyhow::Error> {
        self.client
            .update_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .attribute_updates(
                ROLE,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(serde_json::to_string(&role)?))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn is_user_exist(&self, username: String) -> Result<bool, anyhow::Error> {
        Ok(self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .send()
            .await?
            .item
            .is_some())
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, anyhow::Error> {
        let users = self
            .client
            .scan()
            .table_name(&self.users)
            .send()
            .await?
            .items()
            .iter()
            .map(|i| -> Result<User, anyhow::Error> {
                Ok(User {
                    id: i
                        .get(USER_ID)
                        .context("missing user id")?
                        .as_s()
                        .map_err(|e| anyhow::Error::msg(format!("{:#?}", e)))?
                        .clone(),
                    username: i
                        .get(USERNAME)
                        .context("missing user name")?
                        .as_s()
                        .map_err(|e| anyhow::Error::msg(format!("{:#?}", e)))?
                        .clone(),
                })
            })
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    pub async fn create_user(
        &self,
        user_id: String,
        username: String,
        password_hash: String,
        salt: Vec<u8>,
        is_imported: bool,
        registration_token: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        log::info!("inserting new user: {user_id} / {username}");

        self.client
            .put_item()
            .table_name(&self.users)
            .item(TIMESTAMP, AttributeValue::S(now))
            .item(USER_ID, AttributeValue::S(user_id))
            .item(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .item(PASSWORD_HASH, AttributeValue::S(password_hash))
            .item(SALT, AttributeValue::B(Blob::new(salt)))
            .item(IS_IMPORTED, AttributeValue::Bool(is_imported))
            .item(
                REGISTRATION_TOKEN,
                AttributeValue::S(serde_json::to_string(&registration_token)?),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_password_hash(&self, username: String) -> Result<String, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username.lowercase_trim()))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let password_hash = item
            .get(PASSWORD_HASH)
            .context("must have password")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(password_hash)
    }

    pub async fn create_registration_token(
        &self,
        registration_token: &str,
        role: UserRole,
        single_use: bool,
    ) -> Result<(), anyhow::Error> {
        let utc: DateTime<Utc> = Utc::now();

        self.client
            .put_item()
            .table_name(&self.registration_tokens)
            .item(TOKEN, AttributeValue::S(registration_token.to_owned()))
            .item(ROLE, AttributeValue::S(serde_json::to_string(&role)?))
            .item(TIMESTAMP, AttributeValue::N(utc.timestamp().to_string()))
            .item(ONE_TIME_TOKEN, AttributeValue::Bool(single_use))
            .item(USES, AttributeValue::N(0.to_string()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_registration_token(
        &self,
        registration_token: &str,
    ) -> Result<RegistrationTokenMetadata, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.registration_tokens)
            .key(TOKEN, AttributeValue::S(registration_token.to_owned()))
            .send()
            .await?;

        let item = response.item.context("cannot find registration token")?;

        Ok(RegistrationTokenMetadata {
            role: serde_json::from_str(
                item.get(ROLE)
                    .context("must have role")?
                    .as_s()
                    .map_err(|_| anyhow::Error::msg("not a string"))?,
            )?,
            is_single_use: *item
                .get(ONE_TIME_TOKEN)
                .context("must have one time token field")?
                .as_bool()
                .map_err(|_| anyhow::Error::msg("not a string"))?,
            use_count: item
                .get(USES)
                .context("must have one time token field")?
                .as_n()
                .map_err(|_| anyhow::Error::msg("not a string"))?
                .clone()
                .parse()?,
        })
    }

    pub async fn set_registration_token_use_count(
        &self,
        registration_token: &str,
        count: u32,
    ) -> Result<(), anyhow::Error> {
        self.client
            .update_item()
            .table_name(&self.registration_tokens)
            .key(TOKEN, AttributeValue::S(registration_token.to_owned()))
            .attribute_updates(
                USES,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::N(count.to_string()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_config_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<UserOptionsDatabase, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.user_config)
            .key_condition_expression("#id = :user_id")
            .expression_attribute_names("#id", USER_ID)
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
            .send()
            .await?;

        if resp.items().len() == 1 {
            let item = resp.items().first().unwrap();
            let config: UserOptionsDatabase = serde_dynamo::from_item(
                item[USER_CONFIG].as_m().ok().context("no config")?.clone(),
            )?;

            Ok(config)
        } else {
            bail!("error fetching user config for {}", user_id)
        }
    }

    pub async fn set_config_by_user_id(
        &self,
        user_id: &str,
        user_config: &UserOptionsDatabase,
        user_name: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .put_item()
            .table_name(&self.user_config)
            .item(USER_ID, AttributeValue::S(user_id.to_string()))
            .item(
                USER_CONFIG,
                AttributeValue::M(serde_dynamo::to_item(user_config).unwrap()),
            )
            .item(USER_NAME, AttributeValue::S(user_name.to_string()))
            .send()
            .await?;

        Ok(())
    }
}
