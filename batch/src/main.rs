use crate::{
    jobs::{
        activate_account::ActivateAccountJob, categorise_offers::CategoriseOffersJob,
        create_account::CreateAccountJob, refresh::RefreshJob,
    },
    jwt::validate,
    routes::{
        health::health,
        jobs::{get_jobs, run_job},
    },
    settings::Settings,
    types::ApiState,
};
use axum::{
    extract::MatchedPath,
    http::Request,
    middleware,
    routing::{get, post},
    Router,
};
use base::shutdown::axum_shutdown_signal;
use jobs::job_scheduler::JobScheduler;
use reqwest::Method;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::{log::LevelFilter, Level};

mod jobs;
mod jwt;
mod routes;
mod settings;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().without_time().init();

    let settings = Settings::new()?;

    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let db = Database::connect(opt).await?;
    let scheduler = JobScheduler::new(db);

    let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
        .basic_auth(&settings.proxy.username, &settings.proxy.password);

    let http_client = base::http::get_http_client(proxy)?;

    scheduler
        .add_scheduled(
            RefreshJob {
                http_client: http_client.clone(),
                mcdonalds_config: settings.mcdonalds.clone(),
            },
            "0 */10 * * * *".parse()?,
        )
        .await;

    scheduler
        .add_manual(CreateAccountJob {
            http_client: http_client.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
            email_config: settings.email.clone(),
        })
        .await;

    scheduler
        .add_manual(ActivateAccountJob {
            http_client: http_client.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
            email_config: settings.email.clone(),
        })
        .await;

    scheduler
        .add_scheduled(
            CategoriseOffersJob {
                api_client: openai::ApiClient::new(
                    settings.openai_api_key.clone(),
                    base::http::get_simple_http_client()?,
                ),
            },
            "0 */11 * * * *".parse()?,
        )
        .await;

    tracing::info!("scheduler initializing");
    scheduler.init().await?;

    let handle = scheduler.run().await;

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    let ctrl_c = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    let api_state = ApiState {
        job_scheduler: scheduler.clone(),
        settings,
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(tower_http::cors::Any);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<_>| {
            let matched_path = request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str);

            tracing::info_span!("request", uri = matched_path)
        })
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Millis),
        );

    let health = Router::new()
        .route("/health", get(health))
        .layer(trace_layer.clone());

    let app = Router::new()
        .route("/job", get(get_jobs))
        .route("/job/:job_name", post(run_job))
        .layer(middleware::from_fn_with_state(api_state.clone(), validate))
        .layer(cors)
        .layer(trace_layer)
        .merge(health)
        .with_state(api_state);

    let addr = "[::]:8002".parse::<SocketAddr>().unwrap();
    tracing::info!("starting batch server {addr}");
    let server = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(axum_shutdown_signal());

    tokio::select! {
        _ = ctrl_c => {
            scheduler.shutdown().await;
        },
        _ = terminate => {
            scheduler.shutdown().await;
        },
        _ = server => {
            scheduler.shutdown().await;
        }
    }

    handle.await.map(|r| {
        if let Err(e) = r {
            tracing::error!("error with job scheduler: {}", e)
        }
    })?;
    // FIXME: after cancel, await all remaining tasks with timeout to ensure cleanup is completed

    Ok(())
}
