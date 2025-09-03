use crate::jobs::{
    account_unlock::AccountUnlockJob, activate_account::ActivateAccountJob,
    activate_existing_account::ActivateExistingAccount, categorise_offers::CategoriseOffersJob,
    create_account::CreateAccountJob, generate_recommendations::GenerateRecommendationsJob,
    job_executor::JobExecutor, recategorise_offers::RecategoriseOffersJob, refresh::RefreshJob,
    save_images::SaveImagesJob,
};
use crate::{
    event_manager::EventManager,
    routes::{
        create_event::create_bulk_events, create_event::create_event,
        get_events::get_events_history,
    },
};
use crate::{
    graphql::{
        FinalSchema, MutationRoot, QueryRoot, graphql_handler,
        queries::locations::dataloader::LocationLoader, self_health,
    },
    settings::Settings,
    types::ApiState,
};
use async_graphql::{EmptySubscription, dataloader::DataLoader};
use axum::routing::post;
use axum::{Router, http::Method, routing::get};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use base::{feature_flag::FeatureFlagClient, http::get_http_client};
use caching::{OfferDetailsCache, Redis};
use event_manager::S3BucketType;
use graphql::health;
use graphql::{
    graphiql,
    queries::offers::dataloader::{OfferCountDataLoader, OfferDetailsLoader},
};
use jobs::job_executor;
use reqwest_middleware::ClientWithMiddleware;
use routes::get_events::get_events;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tracing::log::LevelFilter;

mod discord_webhook;
mod event_manager;
mod graphql;
mod jobs;
mod macros;
mod queue;
mod result_extension;
mod routes;
mod settings;
mod types;

const BUCKET_NAME: &str = "maccas-images";

async fn init_job_executor(
    scheduler: JobExecutor,
    settings: Settings,
) -> Result<JobExecutor, anyhow::Error> {
    let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
        .basic_auth(&settings.proxy.username, &settings.proxy.password);

    let http_client = base::http::get_proxied_maccas_http_client(proxy)?;

    let openai_api_client = openai::ApiClient::new(
        settings.openai_api_key.clone(),
        base::http::get_http_client()?,
    );

    scheduler
        .add(RefreshJob {
            http_client: http_client.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
        })
        .await;

    scheduler
        .add(GenerateRecommendationsJob {
            auth_secret: settings.auth_secret.clone(),
            recommendations_api_base: settings.recommendations_api_base.clone(),
        })
        .await;

    scheduler
        .add(CreateAccountJob {
            sensordata_api_base: settings.sensordata_api_base.clone(),
            http_client: http_client.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
            email_config: settings.email.clone(),
        })
        .await;

    scheduler
        .add(ActivateAccountJob {
            http_client: http_client.clone(),
            sensordata_api_base: settings.sensordata_api_base.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
            email_config: settings.email.clone(),
        })
        .await;

    scheduler
        .add(CategoriseOffersJob {
            api_client: openai_api_client.clone(),
        })
        .await;

    scheduler.add(SaveImagesJob).await;

    scheduler.add(AccountUnlockJob).await;

    scheduler
        .add(RecategoriseOffersJob {
            api_client: openai_api_client,
        })
        .await;

    scheduler
        .add(ActivateExistingAccount {
            sensordata_api_base: settings.sensordata_api_base.clone(),
            http_client: http_client.clone(),
            mcdonalds_config: settings.mcdonalds.clone(),
        })
        .await;

    scheduler.init().await?;

    Ok(scheduler)
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    base::tracing::init("api");

    let settings = Settings::new()?;
    let feature_flag_client = FeatureFlagClient::new().await;

    let credentials = s3::creds::Credentials::new(
        Some(&settings.images_bucket.access_key_id),
        Some(&settings.images_bucket.access_secret_key),
        None,
        None,
        None,
    )?;

    let bucket = s3::Bucket::new(
        BUCKET_NAME,
        s3::Region::Custom {
            region: "".to_owned(),
            endpoint: settings.images_bucket.endpoint.clone(),
        },
        credentials,
    )?
    .with_path_style();

    let offer_details_cache =
        if let Some(ref redis_connection_string) = settings.redis_connection_string {
            tracing::info!("redis connection string provided, connecting...");
            let redis = Redis::new(&redis_connection_string).await?;
            Some(OfferDetailsCache::new(redis))
        } else {
            None
        };

    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(30)
        .min_connections(0)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(30))
        .sqlx_logging(false)
        .sqlx_logging_level(LevelFilter::Trace)
        .sqlx_slow_statements_logging_settings(LevelFilter::Off, Duration::from_secs(60));

    let db = Database::connect(opt).await?;

    let job_executor_cancellation_token = CancellationToken::default();

    let event_manager = EventManager::new(db.clone(), 10).await?;
    let job_scheduler = job_executor::JobExecutor::new(
        db.clone(),
        event_manager.clone(),
        job_executor_cancellation_token.clone(),
    )
    .await?;
    let job_executor = init_job_executor(job_scheduler, settings.clone()).await?;

    if let Some(ref offer_details_cache) = offer_details_cache {
        event_manager.set_state::<OfferDetailsCache>(offer_details_cache.clone());
    }
    event_manager.set_state::<Settings>(settings.clone());
    event_manager.set_state::<JobExecutor>(job_executor.clone());
    event_manager.set_state::<S3BucketType>(bucket);
    event_manager.set_state::<ClientWithMiddleware>(get_http_client()?);
    event_manager.set_state::<FeatureFlagClient>(feature_flag_client);

    let job_scheduler_handle = job_executor.run().await;
    let (handle, cancellation_token) = event_manager.process_events();

    let http_client = base::http::get_http_client()?;
    let basic_http_client = base::http::get_basic_http_client()?;

    let schema = FinalSchema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .data(event_manager.clone())
    .data(http_client.clone())
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
            event_manager: event_manager.clone(),
            database: db.clone(),
            cache: offer_details_cache,
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
        // admin only
        .route("/event", post(create_event))
        .route("/event/bulk", post(create_bulk_events))
        .route("/event", get(get_events_history))
        .route("/event/all", get(get_events))
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        // open
        .route("/health", get(health))
        .route("/health/self", get(self_health))
        .layer(cors)
        .with_state(ApiState {
            schema,
            settings,
            event_manager,
        });

    let app = Router::new().nest("/v1", api_routes);

    let addr = "[::]:8000".parse::<SocketAddr>().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::info!("starting api server {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(base::shutdown::axum_shutdown_signal())
        .await?;

    job_executor_cancellation_token.cancel();
    job_executor.shutdown().await;
    cancellation_token.cancel();
    job_scheduler_handle.await??;
    handle.await?;

    Ok(())
}
