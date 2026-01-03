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
use crate::types::{ZakatDetails, ZakatError, WealthType, ErrorDetails, InvalidInputDetails};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::utils::WeightUnit;


use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display, strum::EnumString, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum JewelryUsage {
    #[default]
    Investment,    // Always Zakatable
    PersonalUse,   // Exempt in Shafi/Maliki/Hanbali (Jumhur), Zakatable in Hanafi
}

impl crate::inputs::ToFfiString for JewelryUsage {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl crate::inputs::FromFfiString for JewelryUsage {
    type Err = strum::ParseError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        use std::str::FromStr;
        Self::from_str(s)
    }
}

// MACRO USAGE
crate::zakat_ffi_export! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PreciousMetals {
        pub weight_grams: Decimal,
        pub metal_type: Option<WealthType>,
        pub purity: u32,
        pub usage: JewelryUsage,
    }
}

impl Default for PreciousMetals {
    fn default() -> Self {
        let (liabilities_due_now, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            weight_grams: Decimal::ZERO,
            metal_type: None,
            purity: 24,
            usage: JewelryUsage::Investment,
            liabilities_due_now,
            hawl_satisfied,
            label,
            id,
            acquisition_date,
            _input_errors,
        }
    }
}

impl PreciousMetals {
    // new() is provided by the macro

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
    pub fn weight(mut self, weight: impl IntoZakatDecimal) -> Self {
        match weight.into_zakat_decimal() {
            Ok(v) => self.weight_grams = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    /// Sets the weight using a specific unit.
    pub fn weight_in(mut self, weight: impl IntoZakatDecimal, unit: WeightUnit) -> Self {
         match weight.into_zakat_decimal() {
            Ok(v) => self.weight_grams = unit.to_grams(v),
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    /// Sets the weight in Tola.
    pub fn weight_tola(self, weight: impl IntoZakatDecimal) -> Self {
        self.weight_in(weight, WeightUnit::Tola)
    }

    /// Sets the weight in Troy Ounces.
    pub fn weight_ounce(self, weight: impl IntoZakatDecimal) -> Self {
        self.weight_in(weight, WeightUnit::TroyOunce)
    }

    pub fn metal_type(mut self, metal_type: WealthType) -> Self {
        self.metal_type = Some(metal_type);
        self
    }

    /// Sets gold purity in Karat (1-24) or Silver purity (1-1000).
    ///
    /// If purity is 0 or greater than 1000, the error is collected.
    /// Specific bounds (24 for Gold) are checked during calculation/validation.
    pub fn purity(mut self, purity: u32) -> Self {
        if purity == 0 || purity > 1000 {
            self._input_errors.push(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "purity".to_string(),
                value: purity.to_string(),
                reason_key: "error-invalid-purity".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: Some(self.id),
            })));
        } else {
            self.purity = purity;
        }
        self
    }

    pub fn usage(mut self, usage: JewelryUsage) -> Self {
        self.usage = usage;
        self
    }
}

impl CalculateZakat for PreciousMetals {
    fn validate_input(&self) -> Result<(), ZakatError> { self.validate() }
    fn get_label(&self) -> Option<String> { self.label.clone() }
    fn get_id(&self) -> uuid::Uuid { self.id }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        // Validate deferred input errors first
        self.validate()?;

        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        let metal_type = self.metal_type.clone().ok_or_else(|| 
            ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
                field: "metal_type".to_string(),
                value: "None".to_string(),
                reason_key: "error-type-required".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            }))
        )?;

        if self.weight_grams < Decimal::ZERO {
            return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
                field: "weight".to_string(),
                value: "negative".to_string(),
                reason_key: "error-negative-value".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }

        // Validate Purity Range based on Metal Type
        match metal_type {
            WealthType::Gold => {
                if self.purity > 24 {
                    return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
                        field: "purity".to_string(),
                        value: self.purity.to_string(),
                        reason_key: "error-gold-purity".to_string(),
                        args: None,
                        source_label: self.label.clone(),
                        asset_id: None,
                    })));
                }
            },
            WealthType::Silver => {
                 // Silver purity is usually 0-1000 (millesimal)
                 // No extra check needed as setter checks 0-1000
            },
            _ => return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
                field: "metal_type".to_string(),
                value: format!("{:?}", metal_type),
                reason_key: "error-type-invalid".to_string(),
                args: None, 
                source_label: self.label.clone(),
                asset_id: None,
            }))),
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
            _ => unreachable!(), // Checked above
        };

        if price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                reason_key: "error-price-required".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }

        let nisab_value = ZakatDecimal::new(nisab_threshold_grams)
            .safe_mul(price_per_gram)?
            .with_source(self.label.clone());

        // Override hawl_satisfied if acquisition_date is present
        let hawl_is_satisfied = if let Some(date) = self.acquisition_date {
            crate::hawl::HawlTracker::new(chrono::Local::now().date_naive())
                .acquired_on(date)
                .is_satisfied()
        } else {
            self.hawl_satisfied
        };

        if !hawl_is_satisfied {
            return Ok(ZakatDetails::below_threshold(*nisab_value, metal_type, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        // Normalize weight
        let effective_weight = if metal_type == WealthType::Gold && self.purity < 24 {
            // formula: weight * (karat / 24)
            let purity_ratio = ZakatDecimal::new(Decimal::from(self.purity))
                .safe_div(Decimal::from(24))?.with_source(self.label.clone());
            ZakatDecimal::new(self.weight_grams)
                .safe_mul(*purity_ratio)?.with_source(self.label.clone())
        } else if metal_type == WealthType::Silver && self.purity < 1000 {
            // formula: weight * (purity / 1000)
             let purity_ratio = ZakatDecimal::new(Decimal::from(self.purity))
                .safe_div(Decimal::from(1000))?.with_source(self.label.clone());
            ZakatDecimal::new(self.weight_grams)
                .safe_mul(*purity_ratio)?.with_source(self.label.clone())
        } else {
            ZakatDecimal::new(self.weight_grams)
        };

        let total_value = effective_weight
            .safe_mul(price_per_gram)?
            .with_source(self.label.clone());
        let liabilities = self.liabilities_due_now; // Uses macro field

        // Dynamic rate from strategy (default 2.5%)
        let rate = config.strategy.get_rules().trade_goods_rate;

        // Build calculation trace
        // Build calculation trace
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial("step-weight", "Weight (grams)", self.weight_grams));
        trace.push(crate::types::CalculationStep::initial("step-price-per-gram", "Price per gram", price_per_gram));
        
        if metal_type == crate::types::WealthType::Gold && self.purity < 24 {
             trace.push(crate::types::CalculationStep::info("info-purity-adjustment", format!("Gold Purity Adjustment ({}K / 24K)", self.purity))
                .with_args(std::collections::HashMap::from([("purity".to_string(), self.purity.to_string())])));
             trace.push(crate::types::CalculationStep::result("step-effective-weight", "Effective 24K Weight", *effective_weight));
        } else if metal_type == crate::types::WealthType::Silver && self.purity < 1000 {
             trace.push(crate::types::CalculationStep::info("info-purity-adjustment", format!("Silver Purity Adjustment ({}/1000)", self.purity))
                .with_args(std::collections::HashMap::from([("purity".to_string(), self.purity.to_string())])));
             trace.push(crate::types::CalculationStep::result("step-effective-weight", "Effective Pure Weight", *effective_weight));
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

    #[test]
    fn test_silver_purity_925() {
        let config = ZakatConfig::new().with_silver_price(1.0); // $1/g
        
        // Sterling Silver (925).
        // Nisab = 595g.
        
        // Case 1: 643g * 0.925 = 594.775g < 595g -> Not Payable
        let metal_low = PreciousMetals::new()
            .weight(643.0)
            .metal_type(WealthType::Silver)
            .purity(925)
            .hawl(true);
        let zakat_low = metal_low.calculate_zakat(&config).unwrap();
        assert!(!zakat_low.is_payable);

        // Case 2: 644g * 0.925 = 595.7g > 595g -> Payable
        let metal_high = PreciousMetals::new()
            .weight(644.0)
            .metal_type(WealthType::Silver)
            .purity(925)
            .hawl(true);
        let zakat_high = metal_high.calculate_zakat(&config).unwrap();
        assert!(zakat_high.is_payable);
        
        // Check calculation trace contains purity info
        let trace_str = format!("{:?}", zakat_high.calculation_trace);
        assert!(trace_str.contains("Silver Purity Adjustment"));
    }

    #[test]
    fn test_weight_units_api() {
        let metal_tola = PreciousMetals::new().weight_tola(10.0);
        // 10 Tola * 11.66 = 116.6g
        assert_eq!(metal_tola.weight_grams, dec!(116.6));

        let metal_ounce = PreciousMetals::new().weight_ounce(10.0);
        // 10 Ounce * 31.1034768 = 311.034768g
        assert_eq!(metal_ounce.weight_grams, dec!(311.034768));
    }
}
