pub mod audio;
pub mod image;

/// Resolves the origin URL, optionally routing images through the wsrv.nl
/// compression proxy, mirroring the original Go behaviour.
pub fn origin_url(url: &str, compress_image: bool) -> String {
    if compress_image {
        format!("https://wsrv.nl/?url={url}")
    } else {
        url.to_string()
    }
}
