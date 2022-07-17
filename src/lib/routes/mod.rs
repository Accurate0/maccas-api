use crate::database::Database;

pub mod auth_fallback;
pub mod code;
pub mod deal;
pub mod deals;
pub mod fallback;
pub mod locations;
pub mod points;
pub mod statistics;
pub mod user;

pub struct Context<'a> {
    pub config: crate::config::ApiConfig,
    pub database: Box<dyn Database + Send + Sync + 'a>,
}
