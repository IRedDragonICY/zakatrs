use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use crate::types::ZakatError;
use crate::inputs::IntoZakatDecimal;

use crate::madhab::{Madhab, NisabStandard};

/// Global configuration for Zakat prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatConfig {
    pub madhab: Madhab, // Added field to store the selected Madhab
    pub gold_price_per_gram: Decimal,
    pub silver_price_per_gram: Decimal,
    pub rice_price_per_kg: Option<Decimal>,
    pub rice_price_per_liter: Option<Decimal>,
    
    /// Nisab standard to use for cash, business assets, and investments.
    /// Set automatically via `with_madhab()` or manually via `with_nisab_standard()`.
    pub cash_nisab_standard: NisabStandard,
    
    // Custom Thresholds (Optional override, defaults provided)
    pub nisab_gold_grams: Option<Decimal>, // Default 85g
    pub nisab_silver_grams: Option<Decimal>, // Default 595g
    pub nisab_agriculture_kg: Option<Decimal>, // Default 653kg
}

impl Default for ZakatConfig {
    fn default() -> Self {
        ZakatConfig {
            madhab: Madhab::default(),
            gold_price_per_gram: Decimal::ZERO,
            silver_price_per_gram: Decimal::ZERO,
            rice_price_per_kg: None,
            rice_price_per_liter: None,
            cash_nisab_standard: NisabStandard::default(),
            nisab_gold_grams: None,
            nisab_silver_grams: None,
            nisab_agriculture_kg: None,
        }
    }
}

// Ensure the caller can easily create a config
impl std::str::FromStr for ZakatConfig {
    type Err = ZakatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|e| ZakatError::ConfigurationError(format!("Failed to parse config JSON: {}", e), None))
    }
}

impl ZakatConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates the configuration for logical consistency and safety.
    pub fn validate(&self) -> Result<(), ZakatError> {
        if self.gold_price_per_gram < Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Gold price must be non-negative".to_string(), None));
        }
        if self.silver_price_per_gram < Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price must be non-negative".to_string(), None));
        }

        // Validation Logic based on Nisab Standard
        match self.cash_nisab_standard {
            NisabStandard::Gold => {
                 // Requires Gold price
            }
            NisabStandard::LowerOfTwo => {
                 // Requires both Gold and Silver prices to determine the lower threshold
            }
            _ => {}
        }
        
        if self.cash_nisab_standard == NisabStandard::Gold && self.gold_price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError("Gold price must be > 0 for Gold Nisab Standard".to_string(), None));
        }

        if self.cash_nisab_standard == NisabStandard::Silver && self.silver_price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError("Silver price must be > 0 for Silver Nisab Standard".to_string(), None));
        }

        if self.cash_nisab_standard == NisabStandard::LowerOfTwo {
            if self.gold_price_per_gram <= Decimal::ZERO {
                return Err(ZakatError::MissingConfig { field: "gold_price".to_string(), source: Some("ZakatConfig validation".to_string()) });
            }
            if self.silver_price_per_gram <= Decimal::ZERO {
                return Err(ZakatError::MissingConfig { field: "silver_price".to_string(), source: Some("ZakatConfig validation".to_string()) });
            }
        }

        Ok(())
    }

    /// Attempts to load configuration from environment variables.
    pub fn from_env() -> Result<Self, ZakatError> {
        let gold_str = env::var("ZAKAT_GOLD_PRICE")
            .map_err(|_| ZakatError::ConfigurationError("ZAKAT_GOLD_PRICE env var not set".to_string(), None))?;
        let silver_str = env::var("ZAKAT_SILVER_PRICE")
            .map_err(|_| ZakatError::ConfigurationError("ZAKAT_SILVER_PRICE env var not set".to_string(), None))?;

        let gold_price = gold_str.parse::<Decimal>()
            .map_err(|e| ZakatError::ConfigurationError(format!("Invalid gold price format: {}", e), None))?;
        let silver_price = silver_str.parse::<Decimal>()
            .map_err(|e| ZakatError::ConfigurationError(format!("Invalid silver price format: {}", e), None))?;

        Ok(Self {
            gold_price_per_gram: gold_price,
            silver_price_per_gram: silver_price,
            ..Default::default()
        })
    }

    /// Attempts to load configuration from a JSON file.
    pub fn try_from_json(path: &str) -> Result<Self, ZakatError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ZakatError::ConfigurationError(format!("Failed to read config file: {}", e), None))?;
        
        let config: ZakatConfig = serde_json::from_str(&content)
            .map_err(|e| ZakatError::ConfigurationError(format!("Failed to parse config JSON: {}", e), None))?;
            
        config.validate()?;
        Ok(config)
    }

    /// Creates a ZakatConfig from an async PriceProvider.
    #[cfg(feature = "async")]
    pub async fn from_provider<P: crate::pricing::PriceProvider>(
        provider: &P,
    ) -> Result<Self, ZakatError> {
        let prices = provider.get_prices().await?;
        Ok(Self {
            gold_price_per_gram: prices.gold_per_gram,
            silver_price_per_gram: prices.silver_per_gram,
            ..Default::default()
        })
    }

    /// Refreshes the prices in this configuration using the given provider.
    #[cfg(feature = "async")]
    pub async fn refresh_prices(&mut self, provider: &impl crate::pricing::PriceProvider) -> Result<(), ZakatError> {
        let prices = provider.get_prices().await?;
        self.gold_price_per_gram = prices.gold_per_gram;
        self.silver_price_per_gram = prices.silver_per_gram;
        self.validate()?;
        Ok(())
    }

    // ========== Fluent Helper Methods ========== 
    // These methods allow chaining configuration adjustments.

    pub fn with_gold_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.gold_price_per_gram = p;
        }
        self
    }

    pub fn with_silver_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
             self.silver_price_per_gram = p;
        }
        self
    }

    pub fn with_gold_nisab(mut self, grams: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = grams.into_zakat_decimal() {
            self.nisab_gold_grams = Some(p);
        }
        self
    }

    pub fn with_silver_nisab(mut self, grams: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = grams.into_zakat_decimal() {
            self.nisab_silver_grams = Some(p);
        }
        self
    }

    pub fn with_agriculture_nisab(mut self, kg: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = kg.into_zakat_decimal() {
            self.nisab_agriculture_kg = Some(p);
        }
        self
    }

    pub fn with_rice_price_per_kg(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.rice_price_per_kg = Some(p);
        }
        self
    }

    pub fn with_rice_price_per_liter(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.rice_price_per_liter = Some(p);
        }
        self
    }

    pub fn with_madhab(mut self, madhab: Madhab) -> Self {
        self.madhab = madhab;
        self.cash_nisab_standard = madhab.strategy().get_rules().nisab_standard;
        self
    }

    pub fn with_nisab_standard(mut self, standard: NisabStandard) -> Self {
        self.cash_nisab_standard = standard;
        self
    }

    // Getters
    pub fn get_nisab_gold_grams(&self) -> Decimal {
        use rust_decimal_macros::dec;
        self.nisab_gold_grams.unwrap_or(dec!(85.0))
    }

    pub fn get_nisab_silver_grams(&self) -> Decimal {
        use rust_decimal_macros::dec;
        self.nisab_silver_grams.unwrap_or(dec!(595.0))
    }

    pub fn get_nisab_agriculture_kg(&self) -> Decimal {
        use rust_decimal_macros::dec;
        self.nisab_agriculture_kg.unwrap_or(dec!(653.0))
    }

    pub fn get_monetary_nisab_threshold(&self) -> Decimal {
        let gold_threshold = self.gold_price_per_gram * self.get_nisab_gold_grams();
        let silver_threshold = self.silver_price_per_gram * self.get_nisab_silver_grams();
        
        match self.cash_nisab_standard {
            NisabStandard::Gold => gold_threshold,
            NisabStandard::Silver => silver_threshold,
            NisabStandard::LowerOfTwo => gold_threshold.min(silver_threshold),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_validate_prices() {
        // Zero prices with default settings.
        // Whether this fails depends on the default Madhab/NisabStandard.
        let config = ZakatConfig::new()
            .with_gold_price(dec!(0))
            .with_silver_price(dec!(0));
        let _res = config.validate();
    }
}
