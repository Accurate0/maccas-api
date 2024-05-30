use crate::{
    graphql::{
        graphql_handler, health, queries::locations::dataloader::LocationLoader, self_health,
        FinalSchema, MutationRoot, QueryRoot,
    },
    settings::Settings,
    types::ApiState,
};
use async_graphql::{dataloader::DataLoader, EmptySubscription};
use axum::{http::Method, routing::get, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use base::shutdown::axum_shutdown_signal;
use graphql::{graphiql, queries::offers::dataloader::OfferDetailsLoader};
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing::log::LevelFilter;

mod graphql;
mod macros;
mod settings;
mod types;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    base::tracing::init("api");

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
    .extension(crate::graphql::tracing::Tracing)
    .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(tower_http::cors::Any);

    let api_routes = Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .route("/health", get(health))
        .route("/health/self", get(self_health))
        .layer(cors)
        .with_state(ApiState { schema, settings });

    let app = Router::new().nest("/v1", api_routes);

    let listener = tokio::net::TcpListener::bind("[::]:8000").await.unwrap();
    tracing::info!("starting api server {listener:?}");

    axum::serve(listener, app)
        .with_graceful_shutdown(axum_shutdown_signal())
        .await?;

    Ok(())
}
