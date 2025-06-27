use std::env;
use std::sync::OnceLock;

use opentelemetry::KeyValue;
use opentelemetry::global::{self, BoxedTracer};
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::resource::ResourceDetector;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("dice_server"))
}

fn resource() -> Resource {
    let detectors: Vec<Box<dyn ResourceDetector>> = vec![
        Box::new(OsResourceDetector),
        Box::new(ProcessResourceDetector),
    ];

    Resource::builder()
        .with_detectors(&detectors)
        .with_service_name(env!("CARGO_PKG_NAME"))
        .with_attributes([KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))])
        .build()
}

fn init_tracer_provider() -> SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");

    SdkTracerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build()
}

fn init_logger_provider() -> SdkLoggerProvider {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(exporter)
        .build()
}

fn init_subscriber(tracer_provider: SdkTracerProvider, logger_provider: SdkLoggerProvider) {
    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider).with_filter(filter_otel);

    // let tracer_layer = tracing_opentelemetry::layer()
    //     .with_tracer(tracer_provider.tracer(env!("CARGO_PKG_NAME")))
    //     .with_filter(filter_otel);

    // Uncomment the following lines to enable debug logging to local terminal
    // let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_thread_names(true)
    //     .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(logger_layer)
        // .with(tracer_layer)
        .init()
}

pub fn init_telemetry() {
    let tracer_provider = init_tracer_provider();
    let logger_provider = init_logger_provider();
    init_subscriber(tracer_provider, logger_provider);
}
