use std::convert::Infallible;
use std::sync::OnceLock;

use http_body_util::Full;
use hyper::Method;
use hyper::body::Bytes;
use hyper::{Request, Response};
use opentelemetry::KeyValue;
use opentelemetry::global::{self, BoxedTracer, ObjectSafeTracerProvider};
use opentelemetry::trace::{Span, SpanKind, Status };
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{SdkTracerProvider};
use rand::Rng;
use tracing::{error, info};
// use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::{self, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

fn expensive_operation() {
    info!("Starting expensive operation");
    std::thread::sleep(std::time::Duration::from_millis(100));
    info!("Expensive operation completed");
}

fn get_tracer() -> &'static BoxedTracer {
    static TRACER: OnceLock<BoxedTracer> = OnceLock::new();
    TRACER.get_or_init(|| global::tracer("dice_server"))
}

pub fn init_tracer_provider() {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .expect("Failed to create span exporter");

    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(env!("CARGO_PKG_NAME"))
                .with_attributes([KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    
    let otel_layer = tracing_opentelemetry::layer().;
    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();
    
    global::set_text_map_propagator(TraceContextPropagator::new());
    global::set_tracer_provider(provider);
}

pub fn init_logger_provider() {
    //

    let exporter = opentelemetry_stdout::LogExporter::default();
    let provider = SdkLoggerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(env!("CARGO_PKG_NAME"))
                .with_attributes([KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))])
                .build(),
        )
        .with_batch_exporter(exporter)
        .build();

    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    // let otel_layer = OpenTelemetryTracingBridge::new(&provider).with_filter(filter_otel);
    let otel_layer = tracing_opentelemetry::layer();

    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=debug".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    info!("Logger initialized");
}

async fn roll_dice(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    info!("Received request to roll a dice");
    let random_number = rand::rng().random_range(1..=6);
    info!("Rolled a dice and got: {}", random_number);
    expensive_operation();
    Ok(Response::new(Full::new(Bytes::from(
        random_number.to_string(),
    ))))
}

pub async fn handle(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let tracer = get_tracer();

    let mut span = tracer
        .span_builder(format!("{} {}", req.method(), req.uri().path()))
        .with_kind(SpanKind::Server)
        .start(tracer);

    error!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rolldice") => roll_dice(req).await,
        _ => {
            span.set_status(Status::Ok);
            Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap())
        }
    }
}
