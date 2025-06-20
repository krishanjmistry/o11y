use std::convert::Infallible;

use http_body_util::Full;
use hyper::Method;
use hyper::body::Bytes;
use hyper::{Request, Response};
use opentelemetry::trace::{Span, SpanKind, Status, Tracer};

use rand::Rng;
use tracing::{error, info};

use crate::telemetry::get_tracer;

pub mod telemetry;

fn expensive_operation() {
    info!("Starting expensive operation");
    std::thread::sleep(std::time::Duration::from_millis(100));
    info!("Expensive operation completed");
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
