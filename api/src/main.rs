use async_graphql::{dataloader::DataLoader, EmptyMutation, EmptySubscription, Schema};
use async_graphql_axum::GraphQL;
use axum::{extract::MatchedPath, http::Request, routing::get, Router};
use config::Settings;
use graphql::{graphiql, queries::offers::dataloader::OfferDetailsLoader, QueryRoot};
use sea_orm::Database;
use tower_http::{
    trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

mod config;
mod graphql;
mod utils;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let settings = Settings::new()?;
    let db = Database::connect(settings.database.url).await?;

    let schema = Schema::build(QueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(db.clone())
        .data(DataLoader::new(
            OfferDetailsLoader { database: db },
            tokio::spawn,
        ))
        .finish();

    let app = Router::new()
        .route("/graphql", get(graphiql).post_service(GraphQL::new(schema)))
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
        );

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(utils::axum_shutdown_signal())
        .await?;

    Ok(())
}
