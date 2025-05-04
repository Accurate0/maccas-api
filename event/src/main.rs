use crate::jobs::{
    account_unlock::AccountUnlockJob,
    activate_account::ActivateAccountJob,
    activate_existing_account::ActivateExistingAccount,
    categorise_offers::CategoriseOffersJob,
    create_account::CreateAccountJob,
    generate_recommendations::GenerateRecommendationsJob,
    job_scheduler::{self, JobExecutor},
    recategorise_offers::RecategoriseOffersJob,
    refresh::RefreshJob,
    save_images::SaveImagesJob,
};
use crate::{
    event_manager::EventManager,
    jwt::{validator, validator_admin_only},
    routes::{
        create_event::create_bulk_events, create_event::create_event,
        get_events::get_events_history, health::health,
    },
    settings::Settings,
    state::AppState,
};
use actix_web::middleware::from_fn;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use base::{feature_flag::FeatureFlagClient, http::get_http_client};
use event_manager::S3BucketType;
use reqwest_middleware::ClientWithMiddleware;
use routes::get_events::get_events;
use sea_orm::{ConnectOptions, Database};
use std::{net::SocketAddr, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::log::LevelFilter;

mod discord_webhook;
mod error;
mod event_manager;
mod jobs;
mod jwt;
mod queue;
mod result_extension;
mod routes;
mod settings;
mod state;

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
    base::tracing::init("event");

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
    )?;

    let mut opt = ConnectOptions::new(settings.database.url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let db = Database::connect(opt).await?;

    let job_executor_cancellation_token = CancellationToken::default();

    let event_manager = EventManager::new(db.clone(), 5).await?;
    let job_scheduler = job_scheduler::JobExecutor::new(
        db.clone(),
        event_manager.clone(),
        job_executor_cancellation_token.clone(),
    )
    .await?;
    let job_executor = init_job_executor(job_scheduler, settings.clone()).await?;

    event_manager.set_state::<Settings>(settings.clone());
    event_manager.set_state::<JobExecutor>(job_executor.clone());
    event_manager.set_state::<S3BucketType>(bucket);
    event_manager.set_state::<ClientWithMiddleware>(get_http_client()?);
    event_manager.set_state::<FeatureFlagClient>(feature_flag_client);

    let job_scheduler_handle = job_executor.run().await;
    let (handle, cancellation_token) = event_manager.process_events();

    let addr = "[::]:8001".parse::<SocketAddr>().unwrap();
    tracing::info!("starting event api server {addr}");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                event_manager: event_manager.clone(),
                settings: settings.clone(),
            }))
            .route("/health", web::get().to(health))
            .route(
                "/event",
                web::post()
                    .to(create_event)
                    .wrap(from_fn(validator))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
            .route(
                "/event/bulk",
                web::post()
                    .to(create_bulk_events)
                    .wrap(from_fn(validator))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
            .route(
                "/event",
                web::get()
                    .to(get_events_history)
                    .wrap(from_fn(validator_admin_only))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
            .route(
                "/event/all",
                web::get()
                    .to(get_events)
                    .wrap(from_fn(validator_admin_only))
                    .wrap(RequestTracing::new())
                    .wrap(Logger::default()),
            )
    })
    .bind(addr)?
    .run()
    .await?;

    job_executor_cancellation_token.cancel();
    job_executor.shutdown().await;
    cancellation_token.cancel();
    job_scheduler_handle.await??;
    handle.await?;

    Ok(())
}
