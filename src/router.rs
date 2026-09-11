use std::time::Instant;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::get;
use axum::Router;

use crate::error::AppError;
use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/image/{size}/{media}", get(handlers::image::get_image))
        .route("/audio/{media}", get(handlers::audio::get_audio))
        .fallback(not_found)
        .layer(middleware::from_fn(log_request))
        .with_state(state)
}

async fn log_request(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status();
    let latency_ms = start.elapsed().as_millis();

    if status.is_server_error() {
        tracing::warn!("{method} {uri} {} {latency_ms}ms", status.as_u16());
    } else {
        tracing::info!("{method} {uri} {} {latency_ms}ms", status.as_u16());
    }

    response
}

async fn not_found() -> AppError {
    AppError::BadRequest {
        status: StatusCode::NOT_FOUND,
        message: "not found".into(),
    }
}
