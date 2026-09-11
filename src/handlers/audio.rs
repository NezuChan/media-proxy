use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderName, StatusCode};
use axum::response::Response;

use crate::error::AppError;
use crate::media;
use crate::state::AppState;

const RELAYED_HEADERS: [HeaderName; 7] = [
    header::CONTENT_TYPE,
    header::CONTENT_LENGTH,
    header::CONTENT_RANGE,
    header::ACCEPT_RANGES,
    header::ETAG,
    header::LAST_MODIFIED,
    header::CACHE_CONTROL,
];

pub async fn get_audio(
    State(state): State<AppState>,
    Path(media): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let url = state.cipher.decrypt_hex(&media)?;

    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok());

    let upstream = media::audio::fetch(&state.http, &url, range).await?;

    let status = StatusCode::from_u16(upstream.status().as_u16())
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let mut builder = Response::builder().status(status);
    for name in RELAYED_HEADERS {
        if let Some(value) = upstream.headers().get(&name) {
            builder = builder.header(name, value);
        }
    }

    let stream = upstream.bytes_stream();
    let body = Body::from_stream(stream);

    builder
        .body(body)
        .map_err(|err| AppError::Internal(err.to_string()))
}
