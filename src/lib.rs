use std::convert::Infallible;

use http_body_util::Full;
use hyper::Method;
use hyper::body::Bytes;
use hyper::{Request, Response};

use rand::Rng;
use tracing::{Span, field, info, info_span};

pub mod telemetry;

pub async fn handle(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let root = info_span!(
        parent: None,
        "handle_request",
        method = %req.method(),
        path = %req.uri().path(),
        is_okay = field::Empty,
        "otel.status_code" = "unset",
    );
    let _enter = root.enter();

    info!(name: "my-event-name", target: "my-system", event_id = 20, user_name = "otel", user_email = "otel@opentelemetry.io", message = "This is an example message");

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/rolldice") => roll_dice(req).await,
        _ => {
            root.record("is_okay", false);
            root.record("otel.status_code", "ok");

            Ok(Response::builder()
                .status(404)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap())
        }
    }
}

#[tracing::instrument]
fn expensive_operation() {
    info!("Starting expensive operation");
    std::thread::sleep(std::time::Duration::from_millis(100));
    info!("Expensive operation completed");
}

#[tracing::instrument]
async fn roll_dice(_: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let current_span = Span::current();

    // The following `record`` does nothing to the output as `something_has_gone_wrong` is not declared - it needs to be declared at span declaration
    current_span.record("something_has_gone_wrong", true);

    info!("Received request to roll a dice");
    let random_number = rand::rng().random_range(1..=6);
    info!("Rolled a dice and got: {}", random_number);
    expensive_operation();

    expensive_operation();
    Ok(Response::new(Full::new(Bytes::from(
        random_number.to_string(),
    ))))
}
