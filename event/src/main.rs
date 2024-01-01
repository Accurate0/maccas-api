use crate::{
    event_manager::EventManager, routes::create_event::create_event, settings::Settings,
    state::AppState,
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tracing::log::LevelFilter;

mod error;
mod event_manager;
mod routes;
mod settings;
mod state;

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let settings = Settings::new()?;
    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    // load the task queue immediately with remains from the database
    // aka anything that didn't get processed in the last time
    // figure out from added time and delay when to run it now
    // most likely immediately
    // might need throttling etc, semaphore?

    let db = Database::connect(opt).await?;
    let event_manager = EventManager::new(db);

    event_manager.reload_incomplete_events().await?;
    let (handle, cancellation_token) = event_manager.process_events();

    let addr = "0.0.0.0:8000".parse::<SocketAddr>().unwrap();
    tracing::info!("starting api server {addr}");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                event_manager: event_manager.clone(),
            }))
            .service(create_event)
    })
    .bind(addr)?
    .run()
    .await?;

    cancellation_token.cancel();
    handle.await?;

    Ok(())
}
