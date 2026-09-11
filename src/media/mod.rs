pub mod audio;
pub mod image;

/// Resolves the origin URL, optionally routing images through the wsrv.nl
/// compression proxy. The origin URL must be percent-encoded so that signed
/// query parameters (e.g. `?ex=&is=&hm=`) are not misread as wsrv's own.
pub fn origin_url(url: &str, compress_image: bool) -> String {
    if compress_image {
        format!("https://wsrv.nl/?url={}", urlencoding::encode(url))
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_when_not_compressing() {
        assert_eq!(
            origin_url("https://a/b?c=d&e=f", false),
            "https://a/b?c=d&e=f"
        );
    }

    #[test]
    fn encodes_signed_query_params() {
        let encoded = origin_url("https://cdn.example/a.png?ex=1&is=2&hm=3", true);
        assert_eq!(
            encoded,
            "https://wsrv.nl/?url=https%3A%2F%2Fcdn.example%2Fa.png%3Fex%3D1%26is%3D2%26hm%3D3"
        );
        assert!(!encoded.contains("&is="));
        assert!(!encoded.contains("&hm="));
    }
}
