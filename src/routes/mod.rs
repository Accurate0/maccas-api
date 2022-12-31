use crate::{database::Database, types::config::GeneralConfig};

pub mod admin;
pub mod catchers;
pub mod code;
pub mod deals;
pub mod docs;
pub mod locations;
pub mod points;
pub mod statistics;
pub mod user;

pub struct Context<'a> {
    pub sqs_client: aws_sdk_sqs::Client,
    pub config: GeneralConfig,
    pub database: Box<dyn Database + Send + Sync + 'a>,
    pub authority: Option<aliri_oauth2::Authority>,
}
