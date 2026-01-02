//! Pricing module for Zakat calculations.
//!
//! This module provides abstractions for fetching metal prices from various sources.
//! The core `PriceProvider` trait supports async price fetching, enabling integration
//! with live APIs, databases, or static test data.

use rust_decimal::Decimal;

use crate::types::{ZakatError, InvalidInputDetails};
use crate::inputs::IntoZakatDecimal;

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
/// # Example
/// ```ignore
/// use zakat::pricing::{PriceProvider, Prices, StaticPriceProvider};
///
/// let provider = StaticPriceProvider::new(65.0, 0.85)?;
/// let prices = provider.get_prices().await?;
/// ```
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait PriceProvider {
    /// Fetches current metal prices.
    ///
    /// Returns `Err(ZakatError)` if prices cannot be fetched.
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

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl PriceProvider for StaticPriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        Ok(self.prices.clone())
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
}

#[cfg(feature = "live-pricing")]
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
#[cfg(feature = "live-pricing")]
pub struct BinancePriceProvider {
    client: reqwest::Client,
}

#[cfg(feature = "live-pricing")]
impl BinancePriceProvider {
    /// Creates a new provider with automatic DNS bypass ("Internet Baik" proof).
    ///
    /// This uses a known Cloudfront IP (18.64.23.181) for `api.binance.com` to bypass
    /// local DNS poisoning/blocking commonly found in Indonesia and other regions.
    pub fn new() -> Self {
        // Known Cloudfront IP for api.binance.com (Verified 2026-01-02 via Google DoH)
        // This maps api.binance.com directly to 18.64.23.181, bypassing local DNS.
        let bypass_ip = std::net::SocketAddr::from(([18, 64, 23, 181], 443));
        
        Self {
            client: reqwest::Client::builder()
                .resolve("api.binance.com", bypass_ip)
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
    }
}

#[cfg(feature = "live-pricing")]
impl Default for BinancePriceProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "live-pricing")]
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

        // Warn about missing Silver support

        tracing::warn!("BinancePriceProvider does not support live Silver prices; using fallback/zero");

        Ok(Prices {
            gold_per_gram,
            silver_per_gram: rust_decimal::Decimal::ZERO,
        })
    }
}

#[cfg(all(test, feature = "live-pricing"))]
mod live_tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Ignore by default to avoid spamming the API during CI
    async fn test_binance_live() {
        let provider = BinancePriceProvider::new();
        let prices = provider.get_prices().await.expect("Failed to fetch live prices");
        
        println!("Live Gold Price (Binance): {} USD/g", prices.gold_per_gram);
        
        assert!(prices.gold_per_gram > rust_decimal::Decimal::ZERO);
        assert_eq!(prices.silver_per_gram, rust_decimal::Decimal::ZERO);
    }
}
