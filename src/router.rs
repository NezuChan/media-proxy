use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::error::AppError;
use crate::handlers;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health::health))
        .route("/image/{size}/{media}", get(handlers::image::get_image))
        .route("/audio/{media}", get(handlers::audio::get_audio))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn not_found() -> AppError {
    AppError::BadRequest {
        status: StatusCode::NOT_FOUND,
        message: "not found".into(),
    }
}
