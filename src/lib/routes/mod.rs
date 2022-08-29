use crate::{database::Database, types::config::GeneralConfig};

pub mod admin;
pub mod code;
pub mod deals;
pub mod docs;
pub mod locations;
pub mod points;
pub mod statistics;
pub mod user;

pub struct Context<'a> {
    pub config: GeneralConfig,
    pub database: Box<dyn Database + Send + Sync + 'a>,
}
