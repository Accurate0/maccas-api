use crate::{
    event_manager::EventManager,
    jwt::{validator, validator_admin_only},
    routes::{create_event::create_event, get_events::get_events, health::health},
    settings::Settings,
    state::AppState,
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_lab::middleware::from_fn;
use actix_web_opentelemetry::RequestTracing;
use base::http::get_simple_http_client;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tracing::log::LevelFilter;

mod error;
mod event_manager;
mod jwt;
mod routes;
mod settings;
mod state;

const BUCKET_NAME: &str = "maccas-api-images";

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    base::tracing::init("event");

    let settings = Settings::new()?;

    let credentials = s3::creds::Credentials::new(
        Some(&settings.images_bucket.access_key_id),
        Some(&settings.images_bucket.access_secret_key),
        None,
        None,
        None,
    )?;

    let bucket = s3::Bucket::new(
        BUCKET_NAME,
        s3::Region::Custom {
            region: "apac".to_owned(),
            endpoint: settings.images_bucket.endpoint.clone(),
        },
        credentials,
    )?;

    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let db = Database::connect(opt).await?;

    let event_manager = EventManager::new(db, 5);
    event_manager.set_state(settings.clone());
    event_manager.set_state(bucket);
    event_manager.set_state(get_simple_http_client()?);

    event_manager.reload_incomplete_events().await?;
    let (handle, cancellation_token) = event_manager.process_events();

    let addr = "[::]:8001".parse::<SocketAddr>().unwrap();
    tracing::info!("starting api server {addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                event_manager: event_manager.clone(),
                settings: settings.clone(),
            }))
            .route("/health", web::get().to(health))
            .route(
                "/event",
                web::post()
                    .to(create_event)
                    .wrap(from_fn(validator))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
            .route(
                "/event",
                web::get()
                    .to(get_events)
                    .wrap(from_fn(validator_admin_only))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
    })
    .bind(addr)?
    .run()
    .await?;

    cancellation_token.cancel();
    handle.await?;

    Ok(())
}
