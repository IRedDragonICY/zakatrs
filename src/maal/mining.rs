//! # Fiqh Compliance: Mining & Rikaz
//!
//! ## Classifications
//! - **Rikaz (Buried Treasure)**: Pre-Islamic buried wealth found without labor and extraction cost. Rate is **20% (Khumus)** immediately. No Nisab, No Debt deductions.
//!   - Source: "In Rikaz is the Khumus (one-fifth)." (Sahih Bukhari 1499).
//! - **Ma'adin (Mines)**: Extracted minerals. Treated as gold/silver assets with **2.5%** rate and 85g Gold Nisab. (Subject to Ikhtilaf, default implemented as 2.5%).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MiningType {
    /// Buried Treasure / Ancient Wealth found.
    Rikaz,
    /// Extracted Minerals/Metals from a mine.
    #[default]
    Mines,
}

#[derive(Default)]
pub struct MiningAssets {
    pub value: Decimal,
    pub mining_type: MiningType,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl MiningAssets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl IntoZakatDecimal) -> Self {
        if let Ok(v) = value.into_zakat_decimal() {
            self.value = v;
        }
        self
    }

    pub fn kind(mut self, kind: MiningType) -> Self {
        self.mining_type = kind;
        self
    }

    pub fn debt(mut self, debt: impl IntoZakatDecimal) -> Self {
        if let Ok(d) = debt.into_zakat_decimal() {
            self.liabilities_due_now = d;
        }
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
}

impl CalculateZakat for MiningAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        if self.value < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Mining value must be non-negative".to_string(), self.label.clone()));
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
                    crate::types::CalculationStep::initial("Rikaz Found Value", self.value),
                    crate::types::CalculationStep::info("Rikaz Rule: No Nisab, No Debt Deduction, 20% Rate"),
                    crate::types::CalculationStep::rate("Applied Rate (20%)", rate),
                ];
                
                Ok(ZakatDetails::with_trace(self.value, Decimal::ZERO, Decimal::ZERO, rate, crate::types::WealthType::Rikaz, trace)
                    .with_label(self.label.clone().unwrap_or_default()))
            },
            MiningType::Mines => {
                let nisab_threshold = config.gold_price_per_gram
                    .checked_mul(config.get_nisab_gold_grams())
                    .ok_or(ZakatError::CalculationError("Overflow calculating mining nisab threshold".to_string(), self.label.clone()))?;
                
                // Rate: 2.5%. Nisab: 85g Gold.
                if !self.hawl_satisfied {
                     return Ok(ZakatDetails::below_threshold(nisab_threshold, crate::types::WealthType::Mining, "Hawl (1 lunar year) not met")
                        .with_label(self.label.clone().unwrap_or_default()));
                }
                let rate = dec!(0.025);
                let liabilities = self.liabilities_due_now;

                // Build trace for Mines
                let mut trace = Vec::new();
                trace.push(crate::types::CalculationStep::initial("Extracted Value", self.value));
                trace.push(crate::types::CalculationStep::subtract("Debts Due Now", liabilities));
                let net_val = self.value - liabilities;
                trace.push(crate::types::CalculationStep::result("Net Mining Assets", net_val));
                trace.push(crate::types::CalculationStep::compare("Nisab Threshold (85g Gold)", nisab_threshold));
                
                if net_val >= nisab_threshold && net_val > Decimal::ZERO {
                    trace.push(crate::types::CalculationStep::rate("Applied Rate (2.5%)", rate));
                } else {
                     trace.push(crate::types::CalculationStep::info("Net Value below Nisab - No Zakat Due"));
                }
                
                Ok(ZakatDetails::with_trace(self.value, liabilities, nisab_threshold, rate, crate::types::WealthType::Mining, trace)
                    .with_label(self.label.clone().unwrap_or_default()))
            }
        }
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rikaz() {
        let config = ZakatConfig::default();
        let mining = MiningAssets::new()
            .value(dec!(1000.0))
            .kind(MiningType::Rikaz);
        // Rikaz (Buried Treasure) is taxed at 20% on the gross value.
        // Debts and Hawl are not considered for Rikaz.
        
        let res = mining.debt(dec!(500.0)).hawl(false).calculate_zakat(&config).unwrap();
        // Calculation: 1000 * 0.20 = 200. (Debt of 500 is ignored).
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(200.0));
    }
    
    #[test]
    fn test_minerals() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
         // Nisab 85g = 8500.
         
         let mining = MiningAssets::new()
             .value(dec!(10000.0))
             .kind(MiningType::Mines);
         let res = mining.hawl(true).calculate_zakat(&config).unwrap();
         
         // 10000 > 8500. Rate 2.5%.
         // Due 250.
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, dec!(250.0));
    }
}
