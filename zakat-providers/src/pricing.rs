//! Pricing module for Zakat calculations.
//!
//! This module provides abstractions for fetching metal prices from various sources.
//! The core `PriceProvider` trait supports async price fetching, enabling integration
//! with live APIs, databases, or static test data.
//!
//! ## Platform Support
//! - **Native**: Uses `reqwest` for HTTP requests and `std::time::Instant` for caching
//! - **WASM**: Uses `gloo-net` for HTTP requests and `web-time` for caching

use rust_decimal::Decimal;
use std::sync::{Arc, RwLock};

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use zakat_core::types::{ZakatError, InvalidInputDetails, ErrorDetails};
use zakat_core::inputs::IntoZakatDecimal;

/// Represents current market prices for metals used in Zakat calculations.
#[derive(Debug, Clone, Default)]
pub struct Prices {
    /// Gold price per gram in local currency.
    pub gold_per_gram: Decimal,
    /// Silver price per gram in local currency.
    pub silver_per_gram: Decimal,
}

impl Prices {
    /// Creates a new Prices instance.
    pub fn new(
        gold_per_gram: impl IntoZakatDecimal,
        silver_per_gram: impl IntoZakatDecimal,
    ) -> Result<Self, ZakatError> {
        let gold = gold_per_gram.into_zakat_decimal()?;
        let silver = silver_per_gram.into_zakat_decimal()?;

        if gold < Decimal::ZERO || silver < Decimal::ZERO {
            return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
                field: "prices".to_string(),
                value: "negative".to_string(),
                reason_key: "error-prices-negative".to_string(),
                args: None,
                source_label: None,
                asset_id: None,
            })));
        }

        Ok(Self {
            gold_per_gram: gold,
            silver_per_gram: silver,
        })
    }
}

/// Trait for fetching current metal prices.
///
/// Implementors can fetch prices from various sources:
/// - Static values for testing
/// - Environment variables
/// - REST APIs (Gold API, XE, etc.)
/// - Databases
///
/// ## Platform Notes
/// - On native platforms, implementors must be `Send + Sync`
/// - On WASM, these bounds are relaxed since WASM is single-threaded
#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
pub trait PriceProvider: Send + Sync {
    /// Fetches current metal prices.
    async fn get_prices(&self) -> Result<Prices, ZakatError>;
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
pub trait PriceProvider {
    /// Fetches current metal prices.
    async fn get_prices(&self) -> Result<Prices, ZakatError>;
}

/// A static price provider for testing and development.
///
/// Useful when you want to:
/// - Run unit tests with fixed prices
/// - Demonstrate functionality without live APIs
/// - Use user-provided prices directly
#[derive(Debug, Clone)]
pub struct StaticPriceProvider {
    prices: Prices,
}

impl StaticPriceProvider {
    /// Creates a new StaticPriceProvider with the given prices.
    pub fn new(
        gold_per_gram: impl IntoZakatDecimal,
        silver_per_gram: impl IntoZakatDecimal,
    ) -> Result<Self, ZakatError> {
        Ok(Self {
            prices: Prices::new(gold_per_gram, silver_per_gram)?,
        })
    }

    /// Creates a StaticPriceProvider from an existing Prices instance.
    pub fn from_prices(prices: Prices) -> Self {
        Self { prices }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl PriceProvider for StaticPriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        Ok(self.prices.clone())
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl PriceProvider for StaticPriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        Ok(self.prices.clone())
    }
}

/// A decorator that caches prices for a specified duration.
///
/// This prevents API rate limiting by reusing fetched prices until the TTL expires.
#[derive(Debug, Clone)]
pub struct CachedPriceProvider<P> {
    inner: P,
    cache: Arc<RwLock<Option<(Instant, Prices)>>>,
    ttl: Duration,
}

impl<P> CachedPriceProvider<P> {
    /// Creates a new CachedPriceProvider.
    ///
    /// # Arguments
    /// * `inner` - The price provider to decorate.
    /// * `ttl_seconds` - Time-to-live for the cache in seconds.
    pub fn new(inner: P, ttl_seconds: u64) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(None)),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait::async_trait]
impl<P: PriceProvider + Send + Sync> PriceProvider for CachedPriceProvider<P> {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // fast path: check read lock
        if let Ok(guard) = self.cache.read() {
            if let Some((timestamp, prices)) = &*guard {
                if timestamp.elapsed() < self.ttl {
                    return Ok(prices.clone());
                }
            }
        }

        // Slow path: fetch and update
        let new_prices = self.inner.get_prices().await?;
        
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some((Instant::now(), new_prices.clone()));
        }

        Ok(new_prices)
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl<P: PriceProvider> PriceProvider for CachedPriceProvider<P> {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // fast path: check read lock
        if let Ok(guard) = self.cache.read() {
            if let Some((timestamp, prices)) = &*guard {
                if timestamp.elapsed() < self.ttl {
                    return Ok(prices.clone());
                }
            }
        }

        // Slow path: fetch and update
        let new_prices = self.inner.get_prices().await?;
        
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some((Instant::now(), new_prices.clone()));
        }

        Ok(new_prices)
    }
}

/// Network configuration for live price providers.
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub timeout_seconds: u64,
    #[cfg(not(target_arch = "wasm32"))]
    pub binance_api_ip: Option<std::net::IpAddr>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            #[cfg(not(target_arch = "wasm32"))]
            binance_api_ip: None,
        }
    }
}

// =============================================================================
// Native Implementation (using reqwest)
// =============================================================================

#[cfg(all(feature = "live-pricing", not(target_arch = "wasm32")))]
#[derive(serde::Deserialize)]
struct BinanceTicker {
    #[allow(dead_code)]
    symbol: String,
    price: String,
}

/// A price provider that fetches live gold prices from Binance Public API.
///
/// Use this for testing "live" data without needing an API key.
/// Note: This provider does not support Silver prices (returns 0.0).
#[cfg(all(feature = "live-pricing", not(target_arch = "wasm32")))]
pub struct BinancePriceProvider {
    client: reqwest::Client,
}

#[cfg(all(feature = "live-pricing", not(target_arch = "wasm32")))]
impl BinancePriceProvider {
    /// Creates a new provider with resilient connection logic.
    pub fn new(config: &NetworkConfig) -> Self {
        let mut builder = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds));

        let use_hardcoded = if let Some(ip) = config.binance_api_ip {
             let socket = std::net::SocketAddr::new(ip, 443);
             builder = builder.resolve("api.binance.com", socket);
             false
        } else {
             use std::net::ToSocketAddrs;
             ("api.binance.com", 443).to_socket_addrs().is_err()
        };

        if use_hardcoded {
             tracing::warn!("Binance DNS resolution failed; using hardcoded Cloudfront IP");
             let bypass_ip = std::net::SocketAddr::from(([18, 64, 23, 181], 443));
             builder = builder.resolve("api.binance.com", bypass_ip);
        }

        Self {
            client: builder.build().unwrap_or_default(),
        }
    }
}

#[cfg(all(feature = "live-pricing", not(target_arch = "wasm32")))]
impl Default for BinancePriceProvider {
    fn default() -> Self {
        Self::new(&NetworkConfig::default())
    }
}

#[cfg(all(feature = "live-pricing", not(target_arch = "wasm32")))]
#[async_trait::async_trait]
impl PriceProvider for BinancePriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // 1 Troy Ounce = 31.1034768 Grams
        const OUNCE_TO_GRAM: rust_decimal::Decimal = rust_decimal_macros::dec!(31.1034768);
        
        // Fetch Gold Price (PAXG/USDT)
        let url = "https://api.binance.com/api/v3/ticker/price?symbol=PAXGUSDT";
        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Binance API error: {}", e)))?;
            
        let ticker: BinanceTicker = response.json()
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Failed to parse Binance response: {}", e)))?;
            
        let price_per_ounce = rust_decimal::Decimal::from_str_exact(&ticker.price)
            .map_err(|e| ZakatError::CalculationError(Box::new(ErrorDetails { 
                reason_key: "error-calculation-failed".to_string(),
                args: Some(std::collections::HashMap::from([("details".to_string(), format!("Failed to parse price decimal: {}", e))])),
                source_label: None,
                asset_id: None,
            })))?;
            
        let gold_per_gram = price_per_ounce / OUNCE_TO_GRAM;

        tracing::warn!("BinancePriceProvider does not support live Silver prices; using fallback/zero");

        Ok(Prices {
            gold_per_gram,
            silver_per_gram: rust_decimal::Decimal::ZERO,
        })
    }
}

// =============================================================================
// WASM Implementation (using gloo-net)
// =============================================================================

#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct BinanceTickerWasm {
    #[allow(dead_code)]
    symbol: String,
    price: String,
}

/// A price provider that fetches live gold prices from Binance Public API (WASM version).
///
/// Uses browser's Fetch API through gloo-net for WASM compatibility.
#[cfg(target_arch = "wasm32")]
pub struct BinancePriceProvider;

#[cfg(target_arch = "wasm32")]
impl BinancePriceProvider {
    /// Creates a new WASM-compatible Binance price provider.
    pub fn new(_config: &NetworkConfig) -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for BinancePriceProvider {
    fn default() -> Self {
        Self
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait::async_trait(?Send)]
impl PriceProvider for BinancePriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        use gloo_net::http::Request;
        
        // 1 Troy Ounce = 31.1034768 Grams
        const OUNCE_TO_GRAM: rust_decimal::Decimal = rust_decimal_macros::dec!(31.1034768);
        
        // Fetch Gold Price (PAXG/USDT)
        let url = "https://api.binance.com/api/v3/ticker/price?symbol=PAXGUSDT";
        
        let response = Request::get(url)
            .send()
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Binance API error: {}", e)))?;
            
        let ticker: BinanceTickerWasm = response.json()
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Failed to parse Binance response: {}", e)))?;
            
        let price_per_ounce = rust_decimal::Decimal::from_str_exact(&ticker.price)
            .map_err(|e| ZakatError::CalculationError(Box::new(ErrorDetails { 
                reason_key: "error-calculation-failed".to_string(),
                args: Some(std::collections::HashMap::from([("details".to_string(), format!("Failed to parse price decimal: {}", e))])),
                source_label: None,
                asset_id: None,
            })))?;
            
        let gold_per_gram = price_per_ounce / OUNCE_TO_GRAM;

        Ok(Prices {
            gold_per_gram,
            silver_per_gram: rust_decimal::Decimal::ZERO,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_prices_creation() {
        let prices = Prices::new(65, 1).unwrap();
        assert_eq!(prices.gold_per_gram, dec!(65));
        assert_eq!(prices.silver_per_gram, dec!(1));
    }

    #[test]
    fn test_prices_rejects_negative() {
        let result = Prices::new(-10, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_static_provider_creation() {
        let provider = StaticPriceProvider::new(100, 2).unwrap();
        assert_eq!(provider.prices.gold_per_gram, dec!(100));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_cached_provider() {
        let static_provider = StaticPriceProvider::new(100, 2).unwrap();
        let cached_provider = CachedPriceProvider::new(static_provider, 1);

        let prices1 = cached_provider.get_prices().await.unwrap();
        assert_eq!(prices1.gold_per_gram, dec!(100));

        let prices2 = cached_provider.get_prices().await.unwrap();
        assert_eq!(prices2.gold_per_gram, dec!(100));
    }
}
