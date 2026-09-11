use crate::error::AppError;

/// Fetches the origin audio with an optional `Range` header. Redirects are
/// followed by the underlying client; the upstream status is returned as-is
/// so callers can relay `206 Partial Content` responses.
pub async fn fetch(
    client: &reqwest::Client,
    url: &str,
    range: Option<&str>,
) -> Result<reqwest::Response, AppError> {
    let mut request = client.get(url).header(reqwest::header::ACCEPT, "*/*");

    if let Some(range) = range {
        request = request.header(reqwest::header::RANGE, range);
    }

    let response = request.send().await?;
    let status = response.status();

    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(AppError::Upstream {
            status: status.as_u16(),
        });
    }

    Ok(response)
}
