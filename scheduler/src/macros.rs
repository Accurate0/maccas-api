macro_rules! create_trigger_fn {
    ($event:expr, $name: tt) => {
        #[instrument(skip(scheduler_context))]
        async fn $name(scheduler_context: SchedulerContext, _ctx: CronContext<Local>) {
            tracing::info!("triggered event {}", stringify!($name));
            if let Err(e) = send_event(
                $event,
                &scheduler_context.settings,
                &scheduler_context.http_client,
            )
            .await
            {
                tracing::error!("error creating event: {e}")
            };
        }
    };
}

macro_rules! create_worker {
    ($cron_schedule:tt, $name: ident) => {{
        WorkerBuilder::new(stringify!($name))
            .enable_tracing()
            .layer(LoadShedLayer::new())
            .rate_limit(1, Duration::from_secs(10))
            .backend(CronStream::new_with_timezone(
                Schedule::from_str($cron_schedule)?,
                Local,
            ))
            .build_fn($name)
    }};
}
