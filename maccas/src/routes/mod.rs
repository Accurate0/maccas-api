use crate::{
    database::{
        account::AccountRepository, audit::AuditRepository, offer::OfferRepository,
        point::PointRepository, refresh::RefreshRepository, user::UserRepository,
    },
    types::config::GeneralConfig,
};

pub mod admin;
pub mod auth;
pub mod catchers;
pub mod code;
pub mod deals;
pub mod docs;
pub mod graphql;
pub mod health;
pub mod locations;
pub mod points;
pub mod statistics;
pub mod user;

pub struct Database {
    pub user_repository: UserRepository,
    pub account_repository: AccountRepository,
    pub audit_repository: AuditRepository,
    pub offer_repository: OfferRepository,
    pub point_repository: PointRepository,
    pub refresh_repository: RefreshRepository,
}

impl juniper::Context for Context {}

pub struct Context {
    pub sqs_client: aws_sdk_sqs::Client,
    pub secrets_client: aws_sdk_secretsmanager::Client,
    pub config: GeneralConfig,
    pub database: Database,
}
