use ::event::CreateEvent;
use apalis::prelude::*;
use apalis_cron::CronContext;
use apalis_cron::CronStream;
use apalis_cron::Schedule;
use chrono::Local;
use context::SchedulerContext;
use event::send_event;
use std::str::FromStr;
use std::time::Duration;
use tower::load_shed::LoadShedLayer;
use tracing::instrument;

mod context;
mod event;
#[macro_use]
mod macros;
mod settings;

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::Refresh {},
        delay: Duration::from_secs(5),
    },
    trigger_refresh
);

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::UnlockAllAccounts {},
        delay: Duration::from_secs(5),
    },
    trigger_account_unlock
);

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::CategoriseOffers {},
        delay: Duration::from_secs(5),
    },
    trigger_categorise_offers
);

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::GenerateRecommendations {},
        delay: Duration::from_secs(5),
    },
    trigger_generate_recommendations
);

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::CreateAccount {},
        delay: Duration::from_secs(5),
    },
    trigger_create_account
);

create_trigger_fn!(
    CreateEvent {
        event: ::event::Event::ActivateAccount {},
        delay: Duration::from_secs(5),
    },
    trigger_activate_account
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    base::tracing::init("scheduler");

    let refresh_worker = create_worker!("0 */2 * * * *", trigger_refresh);
    let account_unlock_worker = create_worker!("0 0 0 * * *", trigger_account_unlock);
    let categorise_offers_worker = create_worker!("0 0 0 * * *", trigger_categorise_offers);
    let generate_recommendations_worker =
        create_worker!("0 0 * * * *", trigger_generate_recommendations);
    let create_account_worker = create_worker!("0 */15 * * * *", trigger_create_account);
    let activate_account_worker = create_worker!("0 */5 * * * *", trigger_activate_account);

    tracing::info!("scheduler started");
    Monitor::new()
        .register(refresh_worker)
        .register(account_unlock_worker)
        .register(categorise_offers_worker)
        .register(generate_recommendations_worker)
        .register(create_account_worker)
        .register(activate_account_worker)
        .run()
        .await
        .map_err(Into::into)
}
