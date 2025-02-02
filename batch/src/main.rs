use crate::{
    jobs::{
        activate_account::ActivateAccountJob, categorise_offers::CategoriseOffersJob,
        create_account::CreateAccountJob, refresh::RefreshJob, save_images::SaveImagesJob,
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
use jobs::{
    account_unlock::AccountUnlockJob, job_scheduler::JobScheduler,
    recategorise_offers::RecategoriseOffersJob,
};
use reqwest::Method;
use sea_orm::{ConnectOptions, Database};
use std::{future::IntoFuture, net::SocketAddr, time::Duration};
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
    base::tracing::init("batch");
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

    let http_client = base::http::get_proxied_maccas_http_client(proxy)?;

    let disable_jobs = &settings.disable_jobs;
    tracing::info!("disabling the following jobs: {:?}", disable_jobs);

    let openai_api_client = openai::ApiClient::new(
        settings.openai_api_key.clone(),
        base::http::get_http_client()?,
    );

    scheduler
        .add(
            RefreshJob {
                auth_secret: settings.auth_secret.clone(),
                event_api_base: settings.event_api_base.clone(),
                http_client: http_client.clone(),
                mcdonalds_config: settings.mcdonalds.clone(),
            },
            !disable_jobs.iter().any(|j| j == "refresh"),
        )
        .await;

    scheduler
        .add(
            CreateAccountJob {
                sensordata_api_base: settings.sensordata_api_base.clone(),
                http_client: http_client.clone(),
                mcdonalds_config: settings.mcdonalds.clone(),
                email_config: settings.email.clone(),
            },
            !disable_jobs.iter().any(|j| j == "create-account"),
        )
        .await;

    scheduler
        .add(
            ActivateAccountJob {
                http_client: http_client.clone(),
                sensordata_api_base: settings.sensordata_api_base.clone(),
                mcdonalds_config: settings.mcdonalds.clone(),
                email_config: settings.email.clone(),
            },
            !disable_jobs.iter().any(|j| j == "activate-account"),
        )
        .await;

    scheduler
        .add(
            CategoriseOffersJob {
                api_client: openai_api_client.clone(),
            },
            !disable_jobs.iter().any(|j| j == "categorise-offers"),
        )
        .await;

    scheduler
        .add(
            SaveImagesJob {
                http_client: base::http::get_http_client()?,
                auth_secret: settings.auth_secret.clone(),
                event_api_base: settings.event_api_base.clone(),
            },
            !disable_jobs.iter().any(|j| j == "save-images"),
        )
        .await;

    scheduler
        .add(
            AccountUnlockJob,
            !disable_jobs.iter().any(|j| j == "account-unlock"),
        )
        .await;

    scheduler
        .add(
            RecategoriseOffersJob {
                api_client: openai_api_client,
            },
            !disable_jobs.iter().any(|j| j == "recategorise-offers"),
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

    let health = Router::new().route("/health", get(health));

    let app = Router::new()
        .route("/job", get(get_jobs))
        .route("/job/{job_name}", post(run_job))
        .layer(middleware::from_fn_with_state(api_state.clone(), validate))
        .layer(cors)
        .layer(trace_layer)
        .merge(health)
        .with_state(api_state);

    let addr = "[::]:8002".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("starting batch api server {addr}");
    let server = axum::serve(listener, app)
        .with_graceful_shutdown(axum_shutdown_signal())
        .into_future();

    tokio::select! {
        _ = handle => {
            scheduler.shutdown().await;
        },
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

    Ok(())
}
