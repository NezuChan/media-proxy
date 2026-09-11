use std::env;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub key: Vec<u8>,
    pub iv: Vec<u8>,
    pub max_width: u32,
    pub max_height: u32,
    pub compress_image: bool,
    pub image_quality: u8,
    pub cache_max_bytes: u64,
    pub cache_ttl_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| AppError::Config("PORT must be a valid port number".into()))?;

        let key = env::var("KEY").map_err(|_| AppError::Config("KEY is required".into()))?;
        let iv = env::var("IV").map_err(|_| AppError::Config("IV is required".into()))?;

        let key = key.into_bytes();
        let iv = iv.into_bytes();

        if key.len() != 32 {
            return Err(AppError::Config(format!(
                "KEY must be 32 bytes for AES-256, got {}",
                key.len()
            )));
        }
        if iv.len() != 16 {
            return Err(AppError::Config(format!(
                "IV must be 16 bytes for AES-CBC, got {}",
                iv.len()
            )));
        }

        let max_width = env::var("MAX_WIDTH")
            .unwrap_or_else(|_| "1024".to_string())
            .parse()
            .map_err(|_| AppError::Config("MAX_WIDTH must be a positive integer".into()))?;
        let max_height = env::var("MAX_HEIGHT")
            .unwrap_or_else(|_| "1024".to_string())
            .parse()
            .map_err(|_| AppError::Config("MAX_HEIGHT must be a positive integer".into()))?;

        if max_width == 0 || max_height == 0 {
            return Err(AppError::Config(
                "MAX_WIDTH and MAX_HEIGHT must be greater than zero".into(),
            ));
        }

        let compress_image = env::var("COMPRESS_IMAGE")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let image_quality = env::var("IMAGE_QUALITY")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .map_err(|_| AppError::Config("IMAGE_QUALITY must be between 1 and 100".into()))?;
        if !(1..=100).contains(&image_quality) {
            return Err(AppError::Config(
                "IMAGE_QUALITY must be between 1 and 100".into(),
            ));
        }

        let cache_max_bytes = env::var("CACHE_MAX_BYTES")
            .unwrap_or_else(|_| (64 * 1024 * 1024).to_string())
            .parse()
            .map_err(|_| AppError::Config("CACHE_MAX_BYTES must be a positive integer".into()))?;
        let cache_ttl_secs = env::var("CACHE_TTL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .map_err(|_| AppError::Config("CACHE_TTL_SECS must be a positive integer".into()))?;

        Ok(Self {
            host,
            port,
            key,
            iv,
            max_width,
            max_height,
            compress_image,
            image_quality,
            cache_max_bytes,
            cache_ttl_secs,
        })
    }
}
