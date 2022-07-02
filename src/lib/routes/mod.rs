pub mod code;
pub mod deal;
pub mod deals;
pub mod fallback;
pub mod locations;
pub mod statistics;
pub mod user;

pub struct Context {
    pub config: crate::config::ApiConfig,
    pub dynamodb_client: aws_sdk_dynamodb::Client,
}
