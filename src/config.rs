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
    pub fn new(gold_price: Decimal, silver_price: Decimal) -> Self {
        Self {
            gold_price_per_gram: gold_price,
            silver_price_per_gram: silver_price,
            ..Default::default()
        }
    }
    
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
