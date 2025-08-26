use crate::{
    graphql::{
        FinalSchema, MutationRoot, QueryRoot, graphql_handler, health,
        queries::locations::dataloader::LocationLoader, self_health,
    },
    settings::Settings,
    types::ApiState,
};
use async_graphql::{EmptySubscription, dataloader::DataLoader};
use axum::{Router, http::Method, routing::get};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use base::shutdown::axum_shutdown_signal;
use caching::{OfferDetailsCache, Redis};
use graphql::{
    graphiql,
    queries::offers::dataloader::{OfferCountDataLoader, OfferDetailsLoader},
};
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
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
    opt.max_connections(20)
        .min_connections(0)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let offer_details_caching =
        if let Some(ref redis_connection_string) = settings.redis_connection_string {
            tracing::info!("redis connection string provided, connecting...");
            let redis = Redis::new(&redis_connection_string).await?;
            Some(OfferDetailsCache::new(redis))
        } else {
            None
        };

    let db = Database::connect(opt).await?;
    let http_client = base::http::get_http_client()?;
    let basic_http_client = base::http::get_basic_http_client()?;

    let schema = FinalSchema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(http_client)
    // this client is special, it contains no tracing or retry
    .data(basic_http_client)
    .data(settings.clone())
    .data(db.clone())
    .data(DataLoader::new(
        OfferCountDataLoader {
            database: db.clone(),
        },
        tokio::spawn,
    ))
    .data(DataLoader::new(
        OfferDetailsLoader {
            database: db.clone(),
            cache: offer_details_caching,
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
    // FIXME: health checks are showing up
    // .extension(crate::graphql::tracing::Tracing)
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

    let addr = "[::]:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("starting api server {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(axum_shutdown_signal())
        .await?;

    Ok(())
}
