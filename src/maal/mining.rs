use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub enum MiningType {
    /// Buried Treasure / Ancient Wealth found.
    Rikaz,
    /// Extracted Minerals/Metals from a mine.
    Mines,
}

pub struct MiningAssets {
    pub value: Decimal,
    pub mining_type: MiningType,
    pub deductible_liabilities: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl MiningAssets {
    pub fn new(
        value: impl Into<Decimal>,
        mining_type: MiningType,
    ) -> Result<Self, ZakatError> {
        let val = value.into();

        if val < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Mining value must be non-negative".to_string()));
        }

        Ok(Self {
            value: val,
            mining_type,
            deductible_liabilities: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        })
    }

    pub fn with_debt(mut self, debt: impl Into<Decimal>) -> Self {
        self.deductible_liabilities = debt.into();
        self
    }

    pub fn with_hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl CalculateZakat for MiningAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        match self.mining_type {
            MiningType::Rikaz => {
                // Rate: 20%. No Nisab (or minimal). No Debts deduction.
                // Requirement: "Rikaz Rate: 20% (No Hawl, No Debts deduction)."
                // We IGNORE hawl_satisfied here.
                let rate = dec!(0.20);
                
                // We purposefully IGNORE extra_debts for Rikaz as per requirement.
                // We set liabilities to 0.
                // Nisab: 0 (Paying on whatever is found).
                
                Ok(ZakatDetails::new(self.value, Decimal::ZERO, Decimal::ZERO, rate, crate::types::WealthType::Rikaz)
                    .with_label(self.label.clone().unwrap_or_default()))
            },
            MiningType::Mines => {
                let nisab_threshold = config.gold_price_per_gram * config.get_nisab_gold_grams();

                // Rate: 2.5%. Nisab: 85g Gold.
                if !self.hawl_satisfied {
                     return Ok(ZakatDetails::not_payable(nisab_threshold, crate::types::WealthType::Mining, "Hawl (1 lunar year) not met")
                        .with_label(self.label.clone().unwrap_or_default()));
                }
                let rate = dec!(0.025);
                let liabilities = self.deductible_liabilities;
                
                Ok(ZakatDetails::new(self.value, liabilities, nisab_threshold, rate, crate::types::WealthType::Mining)
                    .with_label(self.label.clone().unwrap_or_default()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rikaz() {
        let config = ZakatConfig::default();
        let mining = MiningAssets::new(dec!(1000.0), MiningType::Rikaz).unwrap();
        // Rikaz: 20%. Deduct debt? 
        // Usually Rikaz is on gross, but let's see implementation.
        // Implementation: (value - debt) * 0.20
        
        let res = mining.with_debt(dec!(500.0)).with_hawl(false).calculate_zakat(&config).unwrap();
        // (1000 - 500) * 0.20 = 500 * 0.2 = 100. -> NO, Debt is ignored!
        // 1000 * 0.20 = 200.
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(200.0));
    }
    
    #[test]
    fn test_minerals() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
         // Nisab 85g = 8500.
         
         let mining = MiningAssets::new(dec!(10000.0), MiningType::Mines).unwrap(); // Changed Mineral to Mines
         let res = mining.with_hawl(true).calculate_zakat(&config).unwrap();
         
         // 10000 > 8500. Rate 2.5%.
         // Due 250.
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, dec!(250.0));
    }
}
