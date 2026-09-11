use std::sync::Arc;
use std::time::Duration;

use crate::cache::{self, MediaCache};
use crate::config::Config;
use crate::crypto::Cipher;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub http: reqwest::Client,
    pub cipher: Cipher,
    pub cache: MediaCache,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, AppError> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .build()?;

        let cipher = Cipher::new(&config.key, &config.iv)?;

        let cache = cache::new(
            config.cache_max_bytes,
            Duration::from_secs(config.cache_ttl_secs),
        );

        Ok(Self {
            config: Arc::new(config),
            http,
            cipher,
            cache,
        })
    }
}
