use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;

use crate::error::AppError;
use crate::media;
use crate::state::AppState;

pub async fn get_image(
    State(state): State<AppState>,
    Path((size, media)): Path<(String, String)>,
) -> Result<Response, AppError> {
    let (width, height) = parse_size(&size)?;

    if width > state.config.max_width || height > state.config.max_height {
        return Err(AppError::BadRequest {
            status: StatusCode::BAD_REQUEST,
            message: "width or height size too large".into(),
        });
    }

    let cache_key = format!("image:{size}:{media}");
    if let Some(cached) = state.cache.get(&cache_key).await {
        return Ok(jpeg_response(cached));
    }

    let url = state.cipher.decrypt_hex(&media)?;
    let origin = media::origin_url(&url, state.config.compress_image);

    let upstream = state.http.get(&origin).send().await?;
    let status = upstream.status();
    if !status.is_success() {
        return Err(AppError::Upstream {
            status: status.as_u16(),
        });
    }

    let bytes = upstream.bytes().await?;
    let quality = state.config.image_quality;

    let processed =
        tokio::task::spawn_blocking(move || media::image::process(&bytes, width, height, quality))
            .await
            .map_err(|err| AppError::Internal(err.to_string()))??;

    let output = Bytes::from(processed);
    state.cache.insert(cache_key, output.clone()).await;

    Ok(jpeg_response(output))
}

fn parse_size(size: &str) -> Result<(u32, u32), AppError> {
    let invalid_width = || AppError::BadRequest {
        status: StatusCode::BAD_REQUEST,
        message: "invalid width size".into(),
    };
    let invalid_height = || AppError::BadRequest {
        status: StatusCode::BAD_REQUEST,
        message: "invalid height size".into(),
    };

    let (width, height) = size.split_once('x').ok_or_else(invalid_width)?;
    let width = width.parse::<u32>().map_err(|_| invalid_width())?;
    let height = height.parse::<u32>().map_err(|_| invalid_height())?;

    if width == 0 || height == 0 {
        return Err(invalid_width());
    }

    Ok((width, height))
}

fn jpeg_response(bytes: Bytes) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/jpeg")],
        bytes,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_size() {
        assert_eq!(parse_size("512x256").unwrap(), (512, 256));
    }

    #[test]
    fn rejects_missing_separator() {
        assert!(parse_size("512").is_err());
    }

    #[test]
    fn rejects_zero() {
        assert!(parse_size("0x10").is_err());
    }

    #[test]
    fn rejects_non_numeric() {
        assert!(parse_size("axb").is_err());
    }
}
