use crate::{
    event_manager::EventManager,
    jwt::{validator, validator_admin_only},
    routes::{
        create_event::create_bulk_events, create_event::create_event, get_events::get_events,
        health::health,
    },
    settings::Settings,
    state::AppState,
};
use actix_web::middleware::from_fn;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use base::{feature_flag::FeatureFlagClient, http::get_http_client};
use event_manager::S3BucketType;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tracing::log::LevelFilter;

mod discord_webhook;
mod error;
mod event_manager;
mod jwt;
mod result_extension;
mod routes;
mod settings;
mod state;

const BUCKET_NAME: &str = "maccas-images";

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    base::tracing::init("event");

    let settings = Settings::new()?;
    let feature_flag_client = FeatureFlagClient::new().await;

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
            region: "".to_owned(),
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

    let event_manager = EventManager::new(db, 5).await?;
    event_manager.set_state::<Settings>(settings.clone());
    event_manager.set_state::<S3BucketType>(bucket);
    event_manager.set_state::<ClientWithMiddleware>(get_http_client()?);
    event_manager.set_state::<FeatureFlagClient>(feature_flag_client);

    event_manager.reload_incomplete_events().await?;
    let (handle, cancellation_token) = event_manager.process_events();

    let addr = "[::]:8001".parse::<SocketAddr>().unwrap();
    tracing::info!("starting event api server {addr}");

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
                "/event/bulk",
                web::post()
                    .to(create_bulk_events)
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
