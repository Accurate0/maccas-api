use crate::{
    config::Settings,
    error::JobError,
    jobs::{refresh::RefreshJob, JobScheduler, JobType},
};
use sea_orm::Database;
use std::time::Duration;
use tokio::signal;
use tracing::{Instrument, Level};

mod config;
mod error;
mod jobs;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let settings = Settings::new()?;
    let db = Database::connect(settings.database.url).await?;

    let scheduler = JobScheduler::new(db);

    scheduler.add(RefreshJob, JobType::Continuous).await;
    scheduler.init().await?;
    scheduler.start().await?;

    let cloned_scheduler = scheduler.clone();
    let span = tracing::span!(Level::INFO, "scheduler_tick");

    // FIXME: move into job scheduler
    tokio::spawn(
        async move {
            loop {
                match cloned_scheduler.tick().await {
                    Ok(_) => {}
                    Err(e) => match e {
                        JobError::SchedulerCancelled => {
                            tracing::warn!("scheduler cancelled");
                            break;
                        }
                        e => tracing::error!("unexpected error while ticking scheduler: {}", e),
                    },
                }
                tokio::time::sleep(Duration::from_secs(2)).await
            }
        }
        .instrument(span),
    );

    tokio::time::sleep(Duration::from_secs(10)).await;
    scheduler.shutdown().await;

    let _ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let _terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // tokio::select! {
    //     _ = ctrl_c => {},
    //     _ = terminate => {},
    // }

    // FIXME: after cancel, await all remaining tasks with timeout to ensure cleanup is completed

    Ok(())
}
