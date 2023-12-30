use crate::{
    graphql::{graphql_handler, MaccasSchema},
    settings::Settings,
    types::ApiState,
};
use async_graphql::{dataloader::DataLoader, EmptyMutation, EmptySubscription};
use axum::{
    extract::MatchedPath,
    http::{Method, Request},
    routing::get,
    Router,
};
use graphql::{graphiql, queries::offers::dataloader::OfferDetailsLoader, QueryRoot};
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

    let schema = MaccasSchema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(settings.clone())
        .data(db.clone())
        .data(DataLoader::new(
            OfferDetailsLoader { database: db },
            tokio::spawn,
        ))
        .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(tower_http::cors::Any);

    let app = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
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
