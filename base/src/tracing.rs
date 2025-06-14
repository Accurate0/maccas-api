use http::{HeaderMap, HeaderValue};
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{BatchConfigBuilder, BatchSpanProcessor, Tracer};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const INGEST_URL: &str = "https://api.axiom.co/v1/traces";

pub fn external_tracer(name: &'static str) -> Tracer {
    let ingest_url = std::env::var("INGEST_URL").unwrap_or_else(|_| INGEST_URL.to_string());
    let token = std::env::var("AXIOM_TOKEN").expect("must have axiom token configured");
    let dataset_name = std::env::var("AXIOM_DATASET").expect("must have axiom dataset configured");

    let mut headers = HeaderMap::<HeaderValue>::with_capacity(3);
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    headers.insert(
        "X-Axiom-Dataset",
        HeaderValue::from_str(&dataset_name).unwrap(),
    );
    headers.insert(
        "User-Agent",
        HeaderValue::from_str(&format!("maccas-api/{}", env!("CARGO_PKG_VERSION"))).unwrap(),
    );

    let tags = vec![
        KeyValue::new(TELEMETRY_SDK_NAME, "external-tracer".to_string()),
        KeyValue::new(TELEMETRY_SDK_VERSION, env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        KeyValue::new(SERVICE_NAME, name),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        ),
    ];

    let resource = Resource::builder_empty().with_attributes(tags).build();

    let batch_config = BatchConfigBuilder::default()
        .with_max_queue_size(20480)
        .build();

    let span_exporter = opentelemetry_otlp::HttpExporterBuilder::default()
        .with_http_client(
            std::thread::spawn(|| {
                reqwest::blocking::ClientBuilder::new()
                    .default_headers(headers)
                    .build()
                    .unwrap()
            })
            .join()
            .unwrap(),
        )
        .with_endpoint(ingest_url)
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(span_exporter)
                .with_batch_config(batch_config)
                .build(),
        )
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer(name);
    global::set_tracer_provider(tracer_provider);

    tracer
}

pub fn init(name: &'static str) {
    let tracer = external_tracer(name);

    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    tracing_subscriber::registry()
        .with(
            Targets::default()
                .with_target("otel::tracing", Level::TRACE)
                .with_target("sea_orm::database", Level::TRACE)
                .with_default(Level::INFO),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}
