use crate::{database::Database, types::config::GeneralConfig};

pub mod admin;
pub mod auth;
pub mod catchers;
pub mod code;
pub mod deals;
pub mod docs;
pub mod health;
pub mod locations;
pub mod points;
pub mod statistics;
pub mod user;

pub struct Context<'a> {
    pub sqs_client: aws_sdk_sqs::Client,
    pub secrets_client: aws_sdk_secretsmanager::Client,
    pub config: GeneralConfig,
    pub database: Box<dyn Database + Send + Sync + 'a>,
}
