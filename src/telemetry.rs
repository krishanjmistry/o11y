use std::sync::OnceLock;

use opentelemetry::KeyValue;
use opentelemetry::global::{self, BoxedTracer};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_resource_detectors::{OsResourceDetector, ProcessResourceDetector};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::resource::ResourceDetector;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("dice_server"))
}

fn get_resource() -> Resource {
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

fn init_tracer_provider() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    global::set_tracer_provider(tracer_provider);
}

fn init_logger_provider() {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create log exporter");

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build();

    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    let otel_layer = OpenTelemetryTracingBridge::new(&logger_provider).with_filter(filter_otel);

    // Uncomment the following lines to enable debug logging to local terminal
    // let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     .with_thread_names(true)
    //     .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        // .with(fmt_layer)
        .init();
}

pub fn init_telemetry() {
    init_tracer_provider();
    init_logger_provider();
    info!("Telemetry initialized");
}
