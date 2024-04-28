use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_semantic_conventions::resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME};
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init(name: &'static str) {
    let axiom_layer = tracing_axiom::builder()
        .with_service_name(name)
        .with_tags(&[
            (SERVICE_NAME, name),
            (
                DEPLOYMENT_ENVIRONMENT,
                if cfg!(debug_assertions) {
                    "development"
                } else {
                    "production"
                },
            ),
        ])
        .layer()
        .unwrap();

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(axiom_layer)
        .init();
}
