use axum::{
    Router,
    extract::MatchedPath,
    http::Request,
    middleware,
    routing::{get, post},
};
use base::shutdown::axum_shutdown_signal;
use engine::RecommendationEngine;
use jwt::validate;
use recommendations::{
    GenerateClusterScores, GenerateClusters, GenerateEmbeddings, GenerateEmbeddingsFor,
};
use reqwest::Method;
use routes::{
    generate::{generate, generate_cluster_scores, generate_clusters, generate_for},
    health::health,
};
use sea_orm::{ConnectOptions, Database};
use settings::Settings;
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    LatencyUnit,
    cors::CorsLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, log::LevelFilter};
use types::ApiState;

mod engine;
mod jwt;
mod routes;
mod settings;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    base::tracing::init("recommendations");
    let settings = Settings::new()?;

    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(20)
        .min_connections(0)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let http_client = base::http::get_http_client()?;
    let openai_api_client = openai::ApiClient::new(settings.openai_api_key.clone(), http_client);

    let db = Database::connect(opt).await?;
    let engine = RecommendationEngine::new(db, openai_api_client, settings.clone());

    let api_state = ApiState { settings, engine };

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
        .route(&GenerateEmbeddings::template_path(), post(generate))
        .route(&GenerateClusters::template_path(), post(generate_clusters))
        .route(
            &GenerateClusterScores::template_path(),
            post(generate_cluster_scores),
        )
        .route(GenerateEmbeddingsFor::template_path(), post(generate_for))
        .layer(middleware::from_fn_with_state(api_state.clone(), validate))
        .layer(cors)
        .layer(trace_layer)
        .merge(health)
        .with_state(api_state);

    let addr = "[::]:8003".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("starting recommendations api server {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(axum_shutdown_signal())
        .await?;

    Ok(())
}
