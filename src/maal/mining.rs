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
    pub nisab_threshold_value: Decimal,
}

impl MiningAssets {
    pub fn new(
        value: Decimal,
        mining_type: MiningType,
        config: &ZakatConfig,
    ) -> Result<Self, ZakatError> {
        // For Rikaz, strictly speaking we might not need gold price if there is no Nisab check (some opinions say minimal amount, but generally 20% on whatever is found).
        // However, for consistency and Mines, we'll take config.
        let nisab = config.gold_price_per_gram * config.get_nisab_gold_grams(); 
        Ok(Self {
            value,
            mining_type,
            nisab_threshold_value: nisab,
        })
    }
}

impl CalculateZakat for MiningAssets {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        match self.mining_type {
            MiningType::Rikaz => {
                // Rate: 20%. No Nisab (or minimal). No Debts deduction.
                // Requirement: "Rikaz Rate: 20% (No Hawl, No Debts deduction)."
                let rate = dec!(0.20);
                
                // We purposefully IGNORE extra_debts for Rikaz as per requirement.
                // We set liabilities to 0.
                // Nisab: 0 (Paying on whatever is found).
                
                Ok(ZakatDetails::new(self.value, Decimal::ZERO, Decimal::ZERO, rate, crate::types::WealthType::Rikaz))
            },
            MiningType::Mines => {
                // Rate: 2.5%. Nisab: 85g Gold.
                let rate = dec!(0.025);
                let nisab_threshold = self.nisab_threshold_value;
                let liabilities = extra_debts.unwrap_or(Decimal::ZERO);
                
                Ok(ZakatDetails::new(self.value, liabilities, nisab_threshold, rate, crate::types::WealthType::Mining))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rikaz() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Found treasure worth 1000.
        // Rate 20% = 200.
        // Debt passed (e.g. 500) should be IGNORED.
        
        let mining = MiningAssets::new(dec!(1000.0), MiningType::Rikaz, &config).unwrap();
        let res = mining.calculate_zakat(Some(dec!(500.0))).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(200.0));
        assert_eq!(res.deductible_liabilities, Decimal::ZERO); // Confirm debt was ignored
    }
    
    #[test]
    fn test_mining() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Value 10000. Debt 1000. Net 9000.
        // Payable. 9000 * 2.5% = 225.
        
        let mining = MiningAssets::new(dec!(10000.0), MiningType::Mines, &config).unwrap();
        let res = mining.calculate_zakat(Some(dec!(1000.0))).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(225.0));
    }
}
