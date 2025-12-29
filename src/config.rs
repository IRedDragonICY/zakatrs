use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Global configuration for Zakat prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatConfig {
    pub gold_price_per_gram: Decimal,
    pub silver_price_per_gram: Decimal,
    pub rice_price_per_kg: Option<Decimal>,
    pub rice_price_per_liter: Option<Decimal>,
    
    // Custom Thresholds (Optional override, defaults provided)
    pub nisab_gold_grams: Option<Decimal>, // Default 85g
    pub nisab_silver_grams: Option<Decimal>, // Default 595g
    pub nisab_agriculture_kg: Option<Decimal>, // Default 653kg
}

impl Default for ZakatConfig {
    fn default() -> Self {
        ZakatConfig {
            gold_price_per_gram: Decimal::ZERO,
            silver_price_per_gram: Decimal::ZERO,
            rice_price_per_kg: None,
            rice_price_per_liter: None,
            nisab_gold_grams: None,
            nisab_silver_grams: None,
            nisab_agriculture_kg: None,
        }
    }
}

// Ensure the caller can easily create a config
impl ZakatConfig {
    pub fn new(gold_price: impl Into<Decimal>, silver_price: impl Into<Decimal>) -> Self {
        Self {
            gold_price_per_gram: gold_price.into(),
            silver_price_per_gram: silver_price.into(),
            ..Default::default()
        }
    }

    // ========== Fluent Builder Methods ==========

    /// Sets a custom gold nisab threshold (default: 85g)
    pub fn with_gold_nisab(mut self, grams: impl Into<Decimal>) -> Self {
        self.nisab_gold_grams = Some(grams.into());
        self
    }

    /// Sets a custom silver nisab threshold (default: 595g)
    pub fn with_silver_nisab(mut self, grams: impl Into<Decimal>) -> Self {
        self.nisab_silver_grams = Some(grams.into());
        self
    }

    /// Sets a custom agriculture nisab threshold (default: 653kg)
    pub fn with_agriculture_nisab(mut self, kg: impl Into<Decimal>) -> Self {
        self.nisab_agriculture_kg = Some(kg.into());
        self
    }

    /// Sets the rice price per kilogram (for Fitrah calculations)
    pub fn with_rice_price_per_kg(mut self, price: impl Into<Decimal>) -> Self {
        self.rice_price_per_kg = Some(price.into());
        self
    }

    /// Sets the rice price per liter (for Fitrah calculations)
    pub fn with_rice_price_per_liter(mut self, price: impl Into<Decimal>) -> Self {
        self.rice_price_per_liter = Some(price.into());
        self
    }

    // ========== Getters with Defaults ==========
    
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
}
