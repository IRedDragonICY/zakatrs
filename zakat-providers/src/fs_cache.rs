//! File System Cache for Offline Support

#[cfg(not(target_arch = "wasm32"))]
use crate::pricing::{PriceProvider, Prices};
#[cfg(not(target_arch = "wasm32"))]
use zakat_core::types::ZakatError;
#[cfg(not(target_arch = "wasm32"))]
use async_trait::async_trait;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(not(target_arch = "wasm32"))]
use serde::{Serialize, Deserialize};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Deserialize)]
struct CachedData {
    timestamp: u64,
    prices: Prices,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct FileSystemPriceCache<P> {
    inner: P,
    ttl: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl<P> FileSystemPriceCache<P> {
    pub fn new(inner: P, ttl: Duration) -> Self {
        Self { inner, ttl }
    }

    fn get_cache_path() -> Option<PathBuf> {
        dirs::home_dir().map(|mut p| {
            p.push(".zakat");
            p.push("prices.json");
            p
        })
    }

    fn load_cache_with_ttl(&self) -> Option<Prices> {
        let path = Self::get_cache_path()?;
        if !path.exists() { return None; }

        let file = std::fs::File::open(&path).ok()?;
        let reader = std::io::BufReader::new(file);
        let cached: CachedData = serde_json::from_reader(reader).ok()?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        if now.saturating_sub(cached.timestamp) > self.ttl.as_secs() {
            return None; // Expired
        }

        Some(cached.prices)
    }

    fn save_cache(prices: &Prices) {
        let Some(path) = Self::get_cache_path() else { return };
        
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        let data = CachedData {
            timestamp: now,
            prices: prices.clone(),
        };

        if let Ok(file) = std::fs::File::create(&path) {
            let _ = serde_json::to_writer(file, &data);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait]
impl<P: PriceProvider + Send + Sync> PriceProvider for FileSystemPriceCache<P> {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // 1. Try Cache
        if let Some(prices) = self.load_cache_with_ttl() {
            tracing::info!("Loaded prices from local file cache");
            return Ok(prices);
        }

        // 2. Try Inner
        match self.inner.get_prices().await {
            Ok(prices) => {
                // 3. Save on success
                Self::save_cache(&prices);
                Ok(prices)
            }
            Err(e) => {
                // 4. If inner fails, maybe return expired cache?
                // For now, adhering to fail-fast or strict TTL.
                // But "Goal: Allow the CLI to run offline if it has successfully fetched prices recently."
                // "Check if file exists and is younger than TTL".
                // If it's OLDER, we fall through to here.
                // If network fails, we could try return expired cache with warning?
                // Requirement says: "if file I/O fails ... log warning".
                // Logic: "Hit: Deserialize ... Miss: Call inner".
                // So if expired -> Miss -> Call Inner.
                Err(e)
            }
        }
    }

    fn name(&self) -> &str {
        "FileSystemPriceCache"
    }
}
