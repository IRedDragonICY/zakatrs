//! # Fiqh Compliance: Mining & Rikaz
//!
//! ## Classifications
//! - **Rikaz (Buried Treasure)**: Pre-Islamic buried wealth found without labor and extraction cost. Rate is **20% (Khumus)** immediately. No Nisab, No Debt deductions.
//!   - Source: "In Rikaz is the Khumus (one-fifth)." (Sahih Bukhari 1499).
//! - **Ma'adin (Mines)**: Extracted minerals. Treated as gold/silver assets with **2.5%** rate and 85g Gold Nisab. (Subject to Ikhtilaf, default implemented as 2.5%).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};

use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MiningType {
    /// Buried Treasure / Ancient Wealth found.
    Rikaz,
    /// Extracted Minerals/Metals from a mine.
    #[default]
    Mines,
}

// MACRO USAGE
crate::zakat_asset! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MiningAssets {
        pub value: Decimal,
        pub mining_type: MiningType,
    }
}

impl Default for MiningAssets {
    fn default() -> Self {
        let (liabilities_due_now, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            value: Decimal::ZERO,
            mining_type: MiningType::default(),
            liabilities_due_now,
            hawl_satisfied,
            label,
            id,
            acquisition_date,
            _input_errors,
        }
    }
}

impl MiningAssets {
    // new() is provided by the macro

    /// Sets the mining value.
    /// 
    /// If the value cannot be converted to a valid decimal, the error is
    /// collected and will be returned by `validate()` or `calculate_zakat()`.
    pub fn value(mut self, value: impl IntoZakatDecimal) -> Self {
        match value.into_zakat_decimal() {
            Ok(v) => self.value = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    pub fn kind(mut self, kind: MiningType) -> Self {
        self.mining_type = kind;
        self
    }
}

impl CalculateZakat for MiningAssets {
    fn validate_input(&self) -> Result<(), ZakatError> { self.validate() }
    fn get_label(&self) -> Option<String> { self.label.clone() }
    fn get_id(&self) -> uuid::Uuid { self.id }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        // Validate deferred input errors first
        self.validate()?;
        
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        if self.value < Decimal::ZERO {
            return Err(ZakatError::InvalidInput {
                field: "value".to_string(),
                value: "negative".to_string(),
                reason: "Mining value must be non-negative".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            });
        }

        match self.mining_type {
            MiningType::Rikaz => {
                // Rate: 20%. No Nisab (or minimal). No Debts deduction.
                // Requirement: "Rikaz Rate: 20% (No Hawl, No Debts deduction)."
                // We IGNORE hawl_satisfied here.
                let rate = dec!(0.20);
                
                // We purposefully IGNORE extra_debts for Rikaz as per requirement.
                // We set liabilities to 0.
                // Nisab: 0 (Paying on whatever is found).
                
                // Calculate Trace
                let trace = vec![
                    crate::types::CalculationStep::initial("step-rikaz-value", "Rikaz Found Value", self.value),
                    crate::types::CalculationStep::info("info-rikaz-rule", "Rikaz Rule: No Nisab, No Debt Deduction, 20% Rate"),
                    crate::types::CalculationStep::rate("step-rate-applied", "Applied Rate (20%)", rate),
                ];
                
                Ok(ZakatDetails::with_trace(self.value, Decimal::ZERO, Decimal::ZERO, rate, crate::types::WealthType::Rikaz, trace)
                    .with_label(self.label.clone().unwrap_or_default()))
            },
            MiningType::Mines => {
                let nisab_threshold = ZakatDecimal::new(config.gold_price_per_gram)
                    .safe_mul(config.get_nisab_gold_grams())?
                    .with_source(self.label.clone());
                
                // Rate: 2.5%. Nisab: 85g Gold.
                // Dynamic rate from strategy (default 2.5%)
                let rate = config.strategy.get_rules().trade_goods_rate;

                let trace_steps = vec![
                    crate::types::CalculationStep::initial("step-extracted-value", "Extracted Value", self.value),
                ];

                let params = MonetaryCalcParams {
                    total_assets: self.value,
                    liabilities: self.liabilities_due_now, // Uses macro field
                    nisab_threshold: *nisab_threshold,
                    rate,
                    wealth_type: crate::types::WealthType::Mining,
                    label: self.label.clone(),
                    hawl_satisfied: self.hawl_satisfied,
                    trace_steps,
                };

                calculate_monetary_asset(params)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZakatConfig;

    #[test]
    fn test_rikaz() {
        let config = ZakatConfig::default();
        let mining = MiningAssets::new()
            .value(1000.0)
            .kind(MiningType::Rikaz);
        // Rikaz (Buried Treasure) is taxed at 20% on the gross value.
        // Debts and Hawl are not considered for Rikaz.
        
        let res = mining.debt(500.0).hawl(false).calculate_zakat(&config).unwrap();
        // Calculation: 1000 * 0.20 = 200. (Debt of 500 is ignored).
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, Decimal::from(200));
    }
    
    #[test]
    fn test_minerals() {
         let config = ZakatConfig::new().with_gold_price(100);
         // Nisab 85g = 8500.
         
         let mining = MiningAssets::new()
             .value(10000.0)
             .kind(MiningType::Mines);
         let res = mining.hawl(true).calculate_zakat(&config).unwrap();
         
         // 10000 > 8500. Rate 2.5%.
         // Due 250.
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, dec!(250));
    }
}
