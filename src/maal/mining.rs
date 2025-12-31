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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MiningType {
    /// Buried Treasure / Ancient Wealth found.
    Rikaz,
    /// Extracted Minerals/Metals from a mine.
    #[default]
    Mines,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningAssets {
    pub value: Decimal,
    pub mining_type: MiningType,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
}

impl MiningAssets {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ..Default::default()
        }
    }

    /// Sets the mining value.
    /// 
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn value(mut self, value: impl IntoZakatDecimal) -> Self {
        self.value = value.into_zakat_decimal()
            .expect("Invalid numeric value for 'value'");
        self
    }

    pub fn kind(mut self, kind: MiningType) -> Self {
        self.mining_type = kind;
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
}

impl CalculateZakat for MiningAssets {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        if self.value < Decimal::ZERO {
            return Err(ZakatError::InvalidInput {
                field: "value".to_string(),
                value: "negative".to_string(),
                reason: "Mining value must be non-negative".to_string(),
                source_label: self.label.clone()
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
                    crate::types::CalculationStep::initial("Rikaz Found Value", self.value),
                    crate::types::CalculationStep::info("Rikaz Rule: No Nisab, No Debt Deduction, 20% Rate"),
                    crate::types::CalculationStep::rate("Applied Rate (20%)", rate),
                ];
                
                Ok(ZakatDetails::with_trace(self.value, Decimal::ZERO, Decimal::ZERO, rate, crate::types::WealthType::Rikaz, trace)
                    .with_label(self.label.clone().unwrap_or_default()))
            },
            MiningType::Mines => {
                let nisab_threshold = ZakatDecimal::new(config.gold_price_per_gram)
                    .safe_mul(config.get_nisab_gold_grams())?
                    .with_source(self.label.clone());
                
                // Rate: 2.5%. Nisab: 85g Gold.
                if !self.hawl_satisfied {
                     return Ok(ZakatDetails::below_threshold(*nisab_threshold, crate::types::WealthType::Mining, "Hawl (1 lunar year) not met")
                        .with_label(self.label.clone().unwrap_or_default()));
                }
                // Dynamic rate from strategy (default 2.5%)
                let rate = config.strategy.get_rules().trade_goods_rate;
                let liabilities = self.liabilities_due_now;

                // Build trace for Mines
                let mut trace = Vec::new();
                trace.push(crate::types::CalculationStep::initial("Extracted Value", self.value));
                trace.push(crate::types::CalculationStep::subtract("Debts Due Now", liabilities));
                let net_val = ZakatDecimal::new(self.value)
                    .safe_sub(liabilities)?
                    .with_source(self.label.clone());
                trace.push(crate::types::CalculationStep::result("Net Mining Assets", *net_val));
                trace.push(crate::types::CalculationStep::compare("Nisab Threshold (85g Gold)", *nisab_threshold));
                
                if *net_val >= *nisab_threshold && *net_val > Decimal::ZERO {
                    trace.push(crate::types::CalculationStep::rate("Applied Trade Goods Rate", rate));
                } else {
                     trace.push(crate::types::CalculationStep::info("Net Value below Nisab - No Zakat Due"));
                }
                
                Ok(ZakatDetails::with_trace(self.value, liabilities, *nisab_threshold, rate, crate::types::WealthType::Mining, trace)
                    .with_label(self.label.clone().unwrap_or_default()))
            }
        }
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
