//! Chain of Responsibility for Price Providers

use crate::pricing::{PriceProvider, Prices};
use zakat_core::types::{ZakatError, ErrorDetails};
use async_trait::async_trait;

/// A provider that chains multiple other providers for resilience.
#[derive(Default)]
pub struct MultiSourcePriceProvider {
    providers: Vec<Box<dyn PriceProvider>>,
}

impl MultiSourcePriceProvider {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn with_provider(mut self, provider: impl PriceProvider + 'static) -> Self {
        self.providers.push(Box::new(provider));
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl PriceProvider for MultiSourcePriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        if self.providers.is_empty() {
             return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: zakat_core::types::ZakatErrorCode::ConfigError,
                reason_key: "no_providers_configured".to_string(),
                source_label: Some("MultiSourcePriceProvider".to_string()),
                suggestion: Some("Add at least one provider to the chain.".to_string()),
                ..Default::default()
            })));
        }

        let mut last_error = None;

        for (idx, provider) in self.providers.iter().enumerate() {
            match provider.get_prices().await {
                Ok(prices) => return Ok(prices),
                Err(e) => {
                    tracing::warn!(
                        "Provider {} ({}) failed: {}. Trying next...", 
                        idx + 1, 
                        provider.name(), 
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ZakatError::NetworkError("All providers failed".to_string())))
    }

    fn name(&self) -> &str {
        "MultiSourcePriceProvider"
    }
}

/// GoldApi.io Provider (Skeleton)
pub struct GoldApiProvider {
    api_key: String,
}

impl GoldApiProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl PriceProvider for GoldApiProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // Mock implementation
        tracing::info!("Mock fetching from GoldApi with key: {}", self.api_key);
        Err(ZakatError::NetworkError("GoldAPI not implemented yet".to_string()))
    }

    fn name(&self) -> &str {
        "GoldApiProvider"
    }
}

/// MetalPriceAPI Provider (Skeleton)
pub struct MetalPriceProvider {
    api_key: String,
}

impl MetalPriceProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl PriceProvider for MetalPriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // Mock implementation
        tracing::info!("Mock fetching from MetalPriceAPI with key: {}", self.api_key);
        Err(ZakatError::NetworkError("MetalPriceAPI not implemented yet".to_string()))
    }

    fn name(&self) -> &str {
        "MetalPriceProvider"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    struct MockProvider {
        name: String,
        should_fail: bool,
        price: Decimal,
    }

    #[async_trait]
    impl PriceProvider for MockProvider {
        async fn get_prices(&self) -> Result<Prices, ZakatError> {
            if self.should_fail {
                Err(ZakatError::NetworkError(format!("{} failed", self.name)))
            } else {
                Ok(Prices {
                    gold_per_gram: self.price,
                    silver_per_gram: Decimal::ZERO,
                })
            }
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_provider_failover() {
        let p1 = MockProvider { name: "P1".to_string(), should_fail: true, price: dec!(100) };
        let p2 = MockProvider { name: "P2".to_string(), should_fail: false, price: dec!(200) };

        let chain = MultiSourcePriceProvider::new()
            .with_provider(p1)
            .with_provider(p2);

        let result = chain.get_prices().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().gold_per_gram, dec!(200));
    }

    #[tokio::test]
    async fn test_all_providers_fail() {
        let p1 = MockProvider { name: "P1".to_string(), should_fail: true, price: dec!(100) };
        let p2 = MockProvider { name: "P2".to_string(), should_fail: true, price: dec!(200) };

        let chain = MultiSourcePriceProvider::new()
            .with_provider(p1)
            .with_provider(p2);

        let result = chain.get_prices().await;
        assert!(result.is_err());
    }
}

