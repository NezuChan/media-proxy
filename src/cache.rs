use std::time::Duration;

use bytes::Bytes;

pub type MediaCache = moka::future::Cache<String, Bytes>;

/// Builds a bounded cache whose capacity is measured in approximate bytes so
/// that large encoded images cannot exhaust process memory.
pub fn new(max_bytes: u64, ttl: Duration) -> MediaCache {
    moka::future::Cache::builder()
        .max_capacity(max_bytes)
        .weigher(|key: &String, value: &Bytes| {
            key.len().saturating_add(value.len()).min(u32::MAX as usize) as u32
        })
        .time_to_live(ttl)
        .build()
}
