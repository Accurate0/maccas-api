pub mod config;
pub mod constants;
pub mod database;
pub mod doc;
pub mod extensions;
pub mod guards;
pub mod logging;
pub mod r#macro;
pub mod proxy;
pub mod rng;
pub mod routes;
pub mod shared;
pub mod types;
pub mod webhook;

#[macro_use]
extern crate rocket;
