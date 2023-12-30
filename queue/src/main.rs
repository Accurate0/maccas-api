use crate::{
    handlers::handle, routes::create_task::create_task, settings::Settings, state::AppState,
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use base::delay_queue::DelayQueue;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::log::LevelFilter;

mod handlers;
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
    let task_queue = DelayQueue::new();
    let cancellation_token = CancellationToken::new();

    let handle = tokio::spawn(handle(
        task_queue.clone(),
        db.clone(),
        cancellation_token.clone(),
    ));

    let addr = "0.0.0.0:8000".parse::<SocketAddr>().unwrap();
    tracing::info!("starting api server {addr}");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                db: db.clone(),
                task_queue: task_queue.clone(),
            }))
            .service(create_task)
    })
    .bind(addr)?
    .run()
    .await?;

    cancellation_token.cancel();

    handle.await?;

    Ok(())
}
