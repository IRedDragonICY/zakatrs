//! # Fiqh Compliance: Precious Metals
//!
//! ## Nisab (Threshold)
//! - **Gold**: 20 Dinars (approx. 85 grams).
//! - **Silver**: 200 Dirhams (approx. 595 grams).
//! - **Source**: Sunan Abu Dawud (1573) and Sahih Muslim (979).
//!
//! ## Jewelry Exemption (Huliyy al-Mubah)
//! This module supports divergent Madhab views via `ZakatStrategy`:
//! - **Shafi'i/Maliki/Hanbali**: Personal permissible jewelry is **EXEMPT** (Reference: *Al-Majmu'* by Al-Nawawi, *Al-Mughni* by Ibn Qudamah).
//! - **Hanafi**: Personal jewelry is **ZAKATABLE** (Reference: *Al-Hidayah* by Al-Marghinani, *Bada'i al-Sana'i* by Al-Kasani).
//!
//! ## Purity Logic
//! - Zakat is due on the *pure* metal content.
//! - Logic: `weight * (karat / 24)` extracts the zakatable 24K equivalent.

use rust_decimal::Decimal;
use crate::types::{ZakatDetails, ZakatError, WealthType};
use crate::traits::{CalculateZakat, ZakatConfigArgument};


use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JewelryUsage {
    #[default]
    Investment,    // Always Zakatable
    PersonalUse,   // Exempt in Shafi/Maliki/Hanbali (Jumhur), Zakatable in Hanafi
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreciousMetals {
    pub weight_grams: Decimal,
    pub metal_type: Option<WealthType>, // Gold or Silver
    pub purity: u32, // Karat for Gold (e.g. 24, 21, 18). Ignored for Silver (assumed pure).
    pub usage: JewelryUsage,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
    // Hidden field for deferred input validation errors
    #[serde(skip)]
    _input_errors: Vec<ZakatError>,
}

impl Default for PreciousMetals {
    fn default() -> Self {
        Self {
            weight_grams: Decimal::ZERO,
            metal_type: None,
            purity: 24,
            usage: JewelryUsage::Investment,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
            id: uuid::Uuid::new_v4(),
            _input_errors: Vec::new(),
        }
    }
}

impl PreciousMetals {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Gold asset with the specified weight in grams.
    /// Defaults to 24K purity, Investment usage, and Hawl satisfied.
    pub fn gold(weight: impl IntoZakatDecimal) -> Self {
        Self::new()
            .weight(weight)
            .metal_type(WealthType::Gold)
            .purity(24)
            .usage(JewelryUsage::Investment)
            .hawl(true)
    }

    /// Creates a Silver asset with the specified weight in grams.
    /// Defaults to Investment usage and Hawl satisfied.
    pub fn silver(weight: impl IntoZakatDecimal) -> Self {
        Self::new()
            .weight(weight)
            .metal_type(WealthType::Silver)
            .usage(JewelryUsage::Investment)
            .hawl(true)
    }

    /// Sets the weight in grams.
    /// 
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn weight(mut self, weight: impl IntoZakatDecimal) -> Self {
        self.weight_grams = weight.into_zakat_decimal()
            .expect("Invalid numeric value for 'weight'");
        self
    }

    pub fn metal_type(mut self, metal_type: WealthType) -> Self {
        self.metal_type = Some(metal_type);
        self
    }

    /// Sets gold purity in Karat (1-24).
    ///
    /// If purity is 0 or greater than 24, the error is collected and will
    /// be returned by `validate()` or `calculate_zakat()`.
    pub fn purity(mut self, purity: u32) -> Self {
        if purity == 0 || purity > 24 {
            self._input_errors.push(ZakatError::InvalidInput {
                field: "purity".to_string(),
                value: purity.to_string(),
                reason: "Gold purity must be between 1 and 24 Karat".to_string(),
                source_label: self.label.clone(),
                asset_id: Some(self.id),
            });
        } else {
            self.purity = purity;
        }
        self
    }

    pub fn usage(mut self, usage: JewelryUsage) -> Self {
        self.usage = usage;
        self
    }

    /// Sets deductible debt.
    /// 
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn debt(mut self, debt: impl IntoZakatDecimal) -> Self {
        self.liabilities_due_now = debt.into_zakat_decimal()
            .expect("Invalid numeric value for 'debt'");
        self
    }

    pub fn hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Restores the asset ID (for database/serialization restoration).
    pub fn with_id(mut self, id: uuid::Uuid) -> Self {
        self.id = id;
        self
    }

    /// Validates the asset and returns any input errors.
    ///
    /// - If no errors, returns `Ok(())`.
    /// - If 1 error, returns `Err(that_error)`.
    /// - If >1 errors, returns `Err(ZakatError::MultipleErrors(...))`.
    pub fn validate(&self) -> Result<(), ZakatError> {
        match self._input_errors.len() {
            0 => Ok(()),
            1 => Err(self._input_errors[0].clone()),
            _ => Err(ZakatError::MultipleErrors(self._input_errors.clone())),
        }
    }
}

impl CalculateZakat for PreciousMetals {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        // Validate deferred input errors first
        self.validate()?;

        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        let metal_type = self.metal_type.clone().ok_or_else(|| 
            ZakatError::InvalidInput { 
                field: "metal_type".to_string(),
                value: "None".to_string(),
                reason: "Metal type must be specified (Gold or Silver)".to_string(), 
                source_label: self.label.clone(),
                asset_id: None,
            }
        )?;

        if self.weight_grams < Decimal::ZERO {
            return Err(ZakatError::InvalidInput { 
                field: "weight".to_string(),
                value: "negative".to_string(),
                reason: "Weight must be non-negative".to_string(), 
                source_label: self.label.clone(),
                asset_id: None,
            });
        }

        match metal_type {
            WealthType::Gold | WealthType::Silver => {},
            _ => return Err(ZakatError::InvalidInput { 
                field: "metal_type".to_string(),
                value: format!("{:?}", metal_type),
                reason: "Type must be Gold or Silver".to_string(), 
                source_label: self.label.clone(),
                asset_id: None,
            }),
        };

        // Check for personal usage exemption first
        if self.usage == JewelryUsage::PersonalUse && config.strategy.get_rules().jewelry_exempt {
             return Ok(ZakatDetails::below_threshold(
                 Decimal::ZERO, 
                 metal_type, 
                 "Exempt per Madhab (Huliyy al-Mubah)"
             ).with_label(self.label.clone().unwrap_or_default()));
        }

        let (price_per_gram, nisab_threshold_grams) = match metal_type {
            WealthType::Gold => (config.gold_price_per_gram, config.get_nisab_gold_grams()),
            WealthType::Silver => (config.silver_price_per_gram, config.get_nisab_silver_grams()),
            _ => return Err(ZakatError::InvalidInput {
                field: "metal_type".to_string(),
                value: format!("{:?}", metal_type),
                reason: "Type must be Gold or Silver".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            }),
        };

        if price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError { 
                reason: "Price for metal not set".to_string(), 
                source_label: self.label.clone(),
                asset_id: None,
            });
        }

        let nisab_value = ZakatDecimal::new(nisab_threshold_grams)
            .safe_mul(price_per_gram)?
            .with_source(self.label.clone());
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(*nisab_value, metal_type, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        // Normalize weight if it's Gold and not 24K
        let effective_weight = if metal_type == WealthType::Gold && self.purity < 24 {
            // formula: weight * (karat / 24)
            let purity_ratio = ZakatDecimal::new(Decimal::from(self.purity))
                .safe_div(Decimal::from(24))?.with_source(self.label.clone());
            ZakatDecimal::new(self.weight_grams)
                .safe_mul(*purity_ratio)?.with_source(self.label.clone())
        } else {
            ZakatDecimal::new(self.weight_grams)
        };

        let total_value = effective_weight
            .safe_mul(price_per_gram)?
            .with_source(self.label.clone());
        let liabilities = self.liabilities_due_now;

        // Dynamic rate from strategy (default 2.5%)
        let rate = config.strategy.get_rules().trade_goods_rate;

        // Build calculation trace
        // Build calculation trace
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial("step-weight", "Weight (grams)", self.weight_grams));
        trace.push(crate::types::CalculationStep::initial("step-price-per-gram", "Price per gram", price_per_gram));
        
        if metal_type == crate::types::WealthType::Gold && self.purity < 24 {
             trace.push(crate::types::CalculationStep::info("info-purity-adjustment", format!("Purity Adjustment ({}K / 24K)", self.purity))
                .with_args(std::collections::HashMap::from([("purity".to_string(), self.purity.to_string())])));
             trace.push(crate::types::CalculationStep::result("step-effective-weight", "Effective 24K Weight", *effective_weight));
        }
        
        trace.push(crate::types::CalculationStep::result("step-total-value", "Total Value", *total_value));
        trace.push(crate::types::CalculationStep::subtract("step-debts-due-now", "Debts Due Now", liabilities));
        
        let net_val = total_value
            .safe_sub(liabilities)?
            .with_source(self.label.clone());
        trace.push(crate::types::CalculationStep::result("step-net-value", "Net Value", *net_val));
        trace.push(crate::types::CalculationStep::compare("step-nisab-check", "Nisab Threshold", *nisab_value));

        if *net_val >= *nisab_value && *net_val > Decimal::ZERO {
            trace.push(crate::types::CalculationStep::rate("step-rate-applied", "Applied Trade Goods Rate", rate));
        } else {
             trace.push(crate::types::CalculationStep::info("status-exempt", "Net Value below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails::with_trace(*total_value, liabilities, *nisab_value, rate, metal_type, trace)
            .with_label(self.label.clone().unwrap_or_default()))
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }

    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::madhab::Madhab;
    use crate::config::ZakatConfig;
    use rust_decimal_macros::dec;

    #[test]
    fn test_gold_below_nisab() {
        let config = ZakatConfig::new().with_gold_price(100);
        let metal = PreciousMetals::new()
            .weight(84.0)
            .metal_type(WealthType::Gold)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        // 84g < 85g -> Not Payable
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_above_nisab() {
        let config = ZakatConfig::new().with_gold_price(100);
        let metal = PreciousMetals::new()
            .weight(85.0)
            .metal_type(WealthType::Gold)
            .hawl(true);
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        // 85g >= 85g -> Payable
        // Value = 8500
        // Due = 8500 * 0.025 = 212.5
        assert!(zakat.is_payable);
        assert_eq!(zakat.zakat_due, dec!(212.5)); // 212.5
    }

    #[test]
    fn test_gold_with_debt() {
         let config = ZakatConfig::new().with_gold_price(100);
        // 100g Gold ($10,000). Debt $2,000. Net $8,000.
        // Nisab 85g = $8,500.
        // Net ($8,000) < Nisab ($8,500) -> Not Payable.
        
        let metal = PreciousMetals::new()
            .weight(100.0)
            .metal_type(WealthType::Gold)
            .debt(2000.0)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_purity_18k() {
        let config = ZakatConfig::new().with_gold_price(100);
        
        // 100g of 18K Gold.
        // Effective Weight = 100 * (18/24) = 75g.
        // Nisab = 85g.
        // 75g < 85g -> Not Payable.
        // If it were treated as 24K, it would be payable.
        
        let metal = PreciousMetals::new()
            .weight(100.0)
            .metal_type(WealthType::Gold)
            .purity(18)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
        
        // Test 24K explicit
        let metal24 = PreciousMetals::new()
            .weight(100.0)
            .metal_type(WealthType::Gold)
            .purity(24)
            .hawl(true);
        let zakat24 = metal24.calculate_zakat(&config).unwrap();
        assert!(zakat24.is_payable);
    }
    #[test]
    fn test_personal_jewelry_hanafi_payable() {
        // Hanafi uses LowerOfTwo. Personal jewelry is Zakatable.
        let config = ZakatConfig::new()
            .with_gold_price(100)
            .with_madhab(Madhab::Hanafi);
        
        // 100g > 85g Nisab
        let metal = PreciousMetals::new()
            .weight(100.0)
            .metal_type(WealthType::Gold)
            .usage(JewelryUsage::PersonalUse)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        assert!(zakat.is_payable);
    }

    #[test]
    fn test_personal_jewelry_shafi_exempt() {
        // Shafi uses Gold Standard. Personal jewelry is Exempt.
        let config = ZakatConfig::new()
            .with_gold_price(100)
            .with_madhab(Madhab::Shafi);
        
        let metal = PreciousMetals::new()
            .weight(100.0)
            .metal_type(WealthType::Gold)
            .usage(JewelryUsage::PersonalUse)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        assert!(!zakat.is_payable);
        assert_eq!(zakat.status_reason, Some("Exempt per Madhab (Huliyy al-Mubah)".to_string()));
    }
}
