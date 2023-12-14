use std::time::Duration;

use crate::{
    config::Settings,
    jobs::{refresh::RefreshJob, JobScheduler},
};
use sea_orm::Database;
use tokio::signal;

mod config;
mod error;
mod jobs;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt().init();

    let settings = Settings::new()?;
    let db = Database::connect(settings.database.url).await?;

    let mut scheduler = JobScheduler::new(db);

    scheduler.add(&RefreshJob {}, jobs::JobType::Continuous);
    scheduler.init().await?;
    scheduler.start().await?;

    tokio::spawn(async move {
        loop {
            scheduler.tick().await;
            tokio::time::sleep(Duration::from_secs(2)).await
        }
    });

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    Ok(())
}
