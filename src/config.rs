use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Islamic school of thought (Madhab) for Zakat calculation.
/// 
/// Each Madhab has different rulings on which Nisab standard to use
/// for cash, paper currency, and trade goods (Urud al-Tijarah).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Madhab {
    /// Hanafi school - Uses LowerOfTwo (Silver or Gold, whichever is lower).
    /// This is considered "Ahwat" (more cautious/beneficial for the poor).
    #[default]
    Hanafi,
    /// Shafi'i school - Uses Gold standard (85g) for monetary wealth.
    Shafi,
    /// Maliki school - Uses Gold standard (85g) for monetary wealth.
    Maliki,
    /// Hanbali school - Uses LowerOfTwo (explicitly prefers benefit for the poor).
    Hanbali,
}

/// Nisab standard for calculating the Zakat threshold on monetary wealth.
/// 
/// - **Gold**: 85 grams of gold equivalent
/// - **Silver**: 595 grams of silver equivalent  
/// - **LowerOfTwo**: Whichever of Gold or Silver produces a lower monetary threshold
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NisabStandard {
    /// Use the gold Nisab (85g × gold_price)
    #[default]
    Gold,
    /// Use the silver Nisab (595g × silver_price)
    Silver,
    /// Use the lower of gold or silver Nisab - most beneficial for the poor
    LowerOfTwo,
}

/// Global configuration for Zakat prices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatConfig {
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

    /// Configures Nisab standard based on Islamic school of thought (Madhab).
    /// 
    /// # Fiqh Basis (verified via web search, 2025-12-29):
    /// - **Hanafi**: Uses LowerOfTwo - "Ahwat" (more beneficial for the poor, dominant opinion)
    /// - **Shafi'i**: Uses Gold standard (traditional position)
    /// - **Maliki**: Uses Gold standard
    /// - **Hanbali**: Uses LowerOfTwo (explicitly prefers benefit for the poor)
    /// 
    /// # Example
    /// ```
    /// use zakat::config::{ZakatConfig, Madhab, NisabStandard};
    /// use rust_decimal_macros::dec;
    /// 
    /// let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
    ///     .with_madhab(Madhab::Hanafi);
    /// 
    /// assert_eq!(config.cash_nisab_standard, NisabStandard::LowerOfTwo);
    /// ```
    pub fn with_madhab(mut self, madhab: Madhab) -> Self {
        self.cash_nisab_standard = match madhab {
            Madhab::Hanafi => NisabStandard::LowerOfTwo,
            Madhab::Shafi => NisabStandard::Gold,
            Madhab::Maliki => NisabStandard::Gold,
            Madhab::Hanbali => NisabStandard::LowerOfTwo,
        };
        self
    }

    /// Manually sets the Nisab standard for cash, business assets, and investments.
    /// 
    /// Use this for custom configurations that don't follow a specific Madhab preset.
    pub fn with_nisab_standard(mut self, standard: NisabStandard) -> Self {
        self.cash_nisab_standard = standard;
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

    /// Calculates the monetary Nisab threshold for cash, business assets, and investments.
    /// 
    /// Returns the threshold value in local currency based on the configured `cash_nisab_standard`:
    /// - **Gold**: `gold_price × 85g`
    /// - **Silver**: `silver_price × 595g`
    /// - **LowerOfTwo**: `min(gold_threshold, silver_threshold)` - most beneficial for the poor
    /// 
    /// # Note
    /// For `LowerOfTwo`, both gold and silver prices must be set for accurate calculation.
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
    fn test_madhab_hanafi_uses_lower_threshold() {
        // Gold: $100/g → $8,500 | Silver: $1/g → $595
        // Hanafi should use LowerOfTwo → $595 (silver is lower)
        let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
            .with_madhab(Madhab::Hanafi);
        
        assert_eq!(config.cash_nisab_standard, NisabStandard::LowerOfTwo);
        assert_eq!(config.get_monetary_nisab_threshold(), dec!(595.0));
    }

    #[test]
    fn test_madhab_shafi_uses_gold_threshold() {
        // Gold: $100/g → $8,500 | Silver: $1/g → $595
        // Shafi should use Gold → $8,500
        let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
            .with_madhab(Madhab::Shafi);
        
        assert_eq!(config.cash_nisab_standard, NisabStandard::Gold);
        assert_eq!(config.get_monetary_nisab_threshold(), dec!(8500.0));
    }

    #[test]
    fn test_madhab_maliki_uses_gold_threshold() {
        let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
            .with_madhab(Madhab::Maliki);
        
        assert_eq!(config.cash_nisab_standard, NisabStandard::Gold);
    }

    #[test]
    fn test_madhab_hanbali_uses_lower_threshold() {
        let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
            .with_madhab(Madhab::Hanbali);
        
        assert_eq!(config.cash_nisab_standard, NisabStandard::LowerOfTwo);
    }

    #[test]
    fn test_lower_of_two_picks_minimum() {
        // Scenario where gold is cheaper (unusual but tests the min logic)
        // Gold: $5/g → $425 | Silver: $1/g → $595
        let config = ZakatConfig::new(dec!(5.0), dec!(1.0))
            .with_madhab(Madhab::Hanafi);
        
        // min(425, 595) = 425
        assert_eq!(config.get_monetary_nisab_threshold(), dec!(425.0));
    }

    #[test]
    fn test_nisab_standard_can_be_set_manually() {
        let config = ZakatConfig::new(dec!(100.0), dec!(1.0))
            .with_nisab_standard(NisabStandard::Silver);
        
        assert_eq!(config.cash_nisab_standard, NisabStandard::Silver);
        assert_eq!(config.get_monetary_nisab_threshold(), dec!(595.0));
    }
}

