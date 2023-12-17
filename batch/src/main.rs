use base::settings::Settings;
use jobs::{job_scheduler::JobScheduler, refresh::RefreshJob};
use log::LevelFilter;
use sea_orm::{ConnectOptions, Database};
use std::time::Duration;
use tokio::signal;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::CLOSE)
        .init();

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
        .add(
            RefreshJob {
                http_client,
                mcdonalds_config: settings.mcdonalds,
            },
            "0 */1 * * * *".parse()?,
        )
        .await;

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

    tokio::select! {
        _ = ctrl_c => {
            scheduler.shutdown().await;
        },
        _ = terminate => {
            scheduler.shutdown().await;
        }
    }

    handle.await?;

    // FIXME: after cancel, await all remaining tasks with timeout to ensure cleanup is completed

    Ok(())
}