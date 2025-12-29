use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BusinessAssets {
    pub cash_on_hand: Decimal,
    pub inventory_value: Decimal,
    pub receivables: Decimal,
    pub short_term_liabilities: Decimal,
}

impl BusinessAssets {
    pub fn new(
        cash: impl Into<Decimal>,
        inventory: impl Into<Decimal>,
        receivables: impl Into<Decimal>,
        short_term_liabilities: impl Into<Decimal>,
    ) -> Self {
        Self {
            cash_on_hand: cash.into(),
            inventory_value: inventory.into(),
            receivables: receivables.into(),
            short_term_liabilities: short_term_liabilities.into(),
        }
    }
}

impl CalculateZakat for BusinessAssets {
    fn calculate_zakat(&self) -> Result<ZakatDetails, ZakatError> {
        // BusinessAssets doesn't hold hawl info itself in this design, 
        // but for safety let's return error as this shouldn't be called directly.
        // Or if we want to allow it, we assume defaults (no debt, hawl satisfied).
        // But the error message below says use Wrapper.
        Err(ZakatError::ConfigurationError("Please use BusinessZakatCalculator wrapper or similar".to_string()))
    }
}

// Better approach to avoid storing state:
// The Trait is fine, but the struct needs the context.
pub struct BusinessZakatCalculator {
    assets: BusinessAssets,
    nisab_threshold_value: Decimal,
    deductible_liabilities: Decimal,
    hawl_satisfied: bool,
}

impl BusinessZakatCalculator {
    pub fn new(assets: BusinessAssets, config: &ZakatConfig) -> Result<Self, ZakatError> {
        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::config::NisabStandard::Silver | crate::config::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Business Nisab".to_string()));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Business Nisab with current standard".to_string()));
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();
        
        Ok(Self {
            assets,
            nisab_threshold_value,
            deductible_liabilities: Decimal::ZERO,
            hawl_satisfied: true,
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
}

impl CalculateZakat for BusinessZakatCalculator {
    fn calculate_zakat(&self) -> Result<ZakatDetails, ZakatError> {
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::not_payable(self.nisab_threshold_value, crate::types::WealthType::Business, "Hawl (1 lunar year) not met"));
        }
        let gross_assets = self.assets.cash_on_hand + self.assets.inventory_value + self.assets.receivables;
        let business_debt = self.assets.short_term_liabilities;
        
        let total_assets = gross_assets;
        
        // Logic: (Assets - Liabilities) >= Nisab
        // ZakatDetails expects:
        // total_assets, deductible_liabilities, nisab values
        
        // total_assets, deductible_liabilities, nisab values
        
        // Sum internal short term liabilities with any extra deductible liabilities set via builder
        let total_liabilities = business_debt + self.deductible_liabilities;

        let rate = dec!(0.025);

        Ok(ZakatDetails::new(total_assets, total_liabilities, self.nisab_threshold_value, rate, crate::types::WealthType::Business))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_zakat() {
        // Gold $100/g -> Nisab $8500
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        
        let assets = BusinessAssets::new(
            dec!(5000.0), // Cash
            dec!(5000.0), // Inventory
            dec!(0.0),    // Receivables
            dec!(1000.0)  // Debt
        );
        // Gross: 10,000. Debt: 1,000. Net: 9,000.
        // Nisab: 8,500.
        // Payable: Yes. 9000 * 2.5% = 225.

        let calculator = BusinessZakatCalculator::new(assets, &config).unwrap();
        let result = calculator.with_hawl(true).calculate_zakat().unwrap();

        assert!(result.is_payable);
        assert_eq!(result.net_assets, dec!(9000.0));
        assert_eq!(result.zakat_due, dec!(225.0));
    }

    #[test]
    fn test_business_below_nisab() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
         let assets = BusinessAssets::new(dec!(1000.0), dec!(1000.0), dec!(0.0), dec!(0.0));
         // Net 2000 < 8500
         
         let calculator = BusinessZakatCalculator::new(assets, &config).unwrap();
         let result = calculator.with_hawl(true).calculate_zakat().unwrap();
         
         assert!(!result.is_payable);
    }

    #[test]
    fn test_business_specific_case() {
        // Test Case: Business Assets 100M, Debt 20M, Nisab 85M (Result: 0).
        // To get Nisab 85M, Gold Price must be 1,000,000 per gram. (85 * 1M = 85M)
        let config = ZakatConfig { gold_price_per_gram: dec!(1000000.0), ..Default::default() };
        
        // Assets 100M
        let assets = BusinessAssets::new(dec!(100000000.0), dec!(0.0), dec!(0.0), dec!(20000000.0));
        // Net = 100M - 20M = 80M.
        // Nisab = 85M.
        // 80M < 85M -> Not Payable.
        
        let calculator = BusinessZakatCalculator::new(assets, &config).unwrap();
        let result = calculator.with_hawl(true).calculate_zakat().unwrap();
        
        assert!(!result.is_payable);
        assert_eq!(result.net_assets, dec!(80000000.0));
    }

    #[test]
    fn test_business_madhab_affects_nisab() {
        use crate::config::Madhab;
        
        // Setup:
        // Gold: $100/g → Nisab = 85 * 100 = $8,500
        // Silver: $2/g -> Nisab = 595 * 2 = $1,190
        // Net Assets: $5,000
        
        // Logic:
        // If Madhab is Shafi (Gold Standard): $5,000 < $8,500 -> Not Payable
        // If Madhab is Hanafi (LowerOfTwo -> Silver): $5,000 > $1,190 -> Payable
        
        let assets = BusinessAssets::new(dec!(5000.0), dec!(0.0), dec!(0.0), dec!(0.0));
        
        // 1. Test Shafi (Gold)
        let shafi_config = ZakatConfig::new(dec!(100.0), dec!(2.0))
            .with_madhab(Madhab::Shafi);
            
        let shafi_calc = BusinessZakatCalculator::new(assets.clone(), &shafi_config).unwrap();
        let shafi_res = shafi_calc.with_hawl(true).calculate_zakat().unwrap();
        
        assert!(!shafi_res.is_payable, "Shafi (Gold) should not be payable as 5000 < 8500");
        assert_eq!(shafi_res.nisab_threshold, dec!(8500.0));

        // 2. Test Hanafi (LowerOfTwo)
        let hanafi_config = ZakatConfig::new(dec!(100.0), dec!(2.0))
            .with_madhab(Madhab::Hanafi);
            
        let hanafi_calc = BusinessZakatCalculator::new(assets, &hanafi_config).unwrap();
        let hanafi_res = hanafi_calc.with_hawl(true).calculate_zakat().unwrap();
        
        assert!(hanafi_res.is_payable, "Hanafi (LowerOfTwo) should be payable as 5000 > 1190");
        assert_eq!(hanafi_res.nisab_threshold, dec!(1190.0));
    }
}
