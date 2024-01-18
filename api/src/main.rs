use crate::{
    graphql::{
        graphql_handler, health, queries::locations::dataloader::LocationLoader, FinalSchema,
        MutationRoot, QueryRoot,
    },
    settings::Settings,
    types::ApiState,
};
use async_graphql::{dataloader::DataLoader, EmptySubscription};
use axum::{
    extract::MatchedPath,
    http::{Method, Request},
    routing::get,
    Router,
};
use graphql::{graphiql, queries::offers::dataloader::OfferDetailsLoader};
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::log::LevelFilter;
use tracing::Level;

mod graphql;
mod macros;
mod settings;
mod types;
mod utils;

#[tokio::main]
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

    let db = Database::connect(opt).await?;
    let http_client = base::http::get_simple_http_client()?;

    let schema = FinalSchema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(http_client)
    .data(settings.clone())
    .data(db.clone())
    .data(DataLoader::new(
        OfferDetailsLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        LocationLoader {
            database: db,
            settings: settings.clone(),
        },
        tokio::spawn,
    ))
    // FIXME: make own logger extension, this one uses info for errors lol
    .extension(async_graphql::extensions::Logger)
    .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/health", get(health))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
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
                ),
        )
        .with_state(ApiState { schema, settings });

    let addr = "0.0.0.0:8000".parse::<SocketAddr>().unwrap();
    tracing::info!("starting api server {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(utils::axum_shutdown_signal())
        .await?;

    Ok(())
}
