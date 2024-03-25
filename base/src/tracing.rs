use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::{
    resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME},
    SCHEMA_URL,
};
use std::env;
use tracing::Level;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub const OTEL_ENDPOINT: &str = "https://otel.highlight.io:4318";

pub fn init(name: &'static str) {
    let resource = Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, name),
            KeyValue::new(
                DEPLOYMENT_ENVIRONMENT,
                if cfg!(debug_assertions) {
                    "develop"
                } else {
                    "production"
                },
            ),
            KeyValue::new(
                "highlight.project_id",
                env::var("OTEL_PROJECT_ID").unwrap_or("".to_owned()),
            ),
        ],
        SCHEMA_URL,
    );

    let otel_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource.clone()),
        )
        .with_batch_config(trace::BatchConfig::default())
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(OTEL_ENDPOINT),
        )
        .install_batch(runtime::Tokio)
        .unwrap();

    let otel_logger = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(opentelemetry_sdk::logs::Config::default().with_resource(resource))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(OTEL_ENDPOINT),
        )
        .install_batch(runtime::Tokio)
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
        .with(OpenTelemetryLayer::new(otel_tracer))
        .with(OpenTelemetryTracingBridge::new(
            &otel_logger.provider().clone(),
        ))
        .init();
}
