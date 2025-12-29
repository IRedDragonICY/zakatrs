use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BusinessAssets {
    pub cash_on_hand: Decimal,
    pub inventory_value: Decimal,
    pub receivables: Decimal,
    pub short_term_liabilities: Decimal,
}

impl BusinessAssets {
    pub fn new(
        cash: impl IntoZakatDecimal,
        inventory: impl IntoZakatDecimal,
        receivables: impl IntoZakatDecimal,
        short_term_liabilities: impl IntoZakatDecimal,
    ) -> Result<Self, ZakatError> {
        let cash_dec = cash.into_zakat_decimal()?;
        let inventory_dec = inventory.into_zakat_decimal()?;
        let receivables_dec = receivables.into_zakat_decimal()?;
        let liabilities_dec = short_term_liabilities.into_zakat_decimal()?;

        if cash_dec < Decimal::ZERO || inventory_dec < Decimal::ZERO || receivables_dec < Decimal::ZERO || liabilities_dec < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Business assets and liabilities must be non-negative".to_string()));
        }

        Ok(Self {
            cash_on_hand: cash_dec,
            inventory_value: inventory_dec,
            receivables: receivables_dec,
            short_term_liabilities: liabilities_dec,
        })
    }
}

impl CalculateZakat for BusinessAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // Delegate to the full calculator with default assumptions:
        // 1. Hawl is satisfied (default for raw assets unless specified)
        // 2. Extra liabilities due now are 0 (BusinessAssets has internal liabilities already)
        // 3. No label (unless we add label to BusinessAssets, but it's not there yet)
        let calculator = BusinessZakatCalculator::new(*self)
            .with_hawl(true);
        
        calculator.calculate_zakat(config)
    }
}

// Better approach to avoid storing state:
// The Trait is fine, but the struct needs the context.
pub struct BusinessZakatCalculator {
    assets: BusinessAssets,
    liabilities_due_now: Decimal,
    hawl_satisfied: bool,
    label: Option<String>,
}

impl BusinessZakatCalculator {
    pub fn new(assets: BusinessAssets) -> Self {
        Self {
            assets,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        }
    }

    pub fn with_debt_due_now(mut self, debt: impl IntoZakatDecimal) -> Result<Self, ZakatError> {
        self.liabilities_due_now = debt.into_zakat_decimal()?;
        Ok(self)
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

impl CalculateZakat for BusinessZakatCalculator {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Business Nisab".to_string()));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Business Nisab with current standard".to_string()));
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(nisab_threshold_value, crate::types::WealthType::Business, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        let gross_assets = self.assets.cash_on_hand + self.assets.inventory_value + self.assets.receivables;
        let business_debt = self.assets.short_term_liabilities;
        
        let total_assets = gross_assets;
        
        // Sum internal short term liabilities with any extra deductible liabilities set via builder
        let total_liabilities = business_debt + self.liabilities_due_now;

        let rate = dec!(0.025);

        Ok(ZakatDetails::new(total_assets, total_liabilities, nisab_threshold_value, rate, crate::types::WealthType::Business)
            .with_label(self.label.clone().unwrap_or_default()))
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
        ).expect("Valid assets");
        // Gross: 10,000. Debt: 1,000. Net: 9,000.
        // Nisab: 8,500.
        // Payable: Yes. 9000 * 2.5% = 225.

        let calculator = BusinessZakatCalculator::new(assets);
        let result = calculator.with_hawl(true).calculate_zakat(&config).unwrap();

        assert!(result.is_payable);
        assert_eq!(result.net_assets, dec!(9000.0));
        assert_eq!(result.zakat_due, dec!(225.0));
    }

    #[test]
    fn test_business_below_nisab() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
         let assets = BusinessAssets::new(dec!(1000.0), dec!(1000.0), dec!(0.0), dec!(0.0)).expect("Valid");
         // Net 2000 < 8500
         
         let calculator = BusinessZakatCalculator::new(assets);
         let result = calculator.with_hawl(true).calculate_zakat(&config).unwrap();
         
         assert!(!result.is_payable);
    }

    #[test]
    fn test_business_specific_case() {
        // Test Case: Business Assets 100M, Debt 20M, Nisab 85M (Result: 0).
        // To get Nisab 85M, Gold Price must be 1,000,000 per gram. (85 * 1M = 85M)
        let config = ZakatConfig { gold_price_per_gram: dec!(1000000.0), ..Default::default() };
        
        // Assets 100M
        let assets = BusinessAssets::new(dec!(100000000.0), dec!(0.0), dec!(0.0), dec!(20000000.0)).expect("Valid");
        // Net = 100M - 20M = 80M.
        // Nisab = 85M.
        // 80M < 85M -> Not Payable.
        
        let calculator = BusinessZakatCalculator::new(assets);
        let result = calculator.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!result.is_payable);
        assert_eq!(result.net_assets, dec!(80000000.0));
    }

    #[test]
    fn test_business_madhab_affects_nisab() {
        use crate::madhab::Madhab;
        
        // Setup:
        // Gold: $100/g → Nisab = 85 * 100 = $8,500
        // Silver: $2/g -> Nisab = 595 * 2 = $1,190
        // Net Assets: $5,000
        
        // Logic:
        // If Madhab is Shafi (Gold Standard): $5,000 < $8,500 -> Not Payable
        // If Madhab is Hanafi (LowerOfTwo -> Silver): $5,000 > $1,190 -> Payable
        
        let assets = BusinessAssets::new(dec!(5000.0), dec!(0.0), dec!(0.0), dec!(0.0)).expect("Valid");
        
        // 1. Test Shafi (Gold)
        let shafi_config = ZakatConfig::new(dec!(100.0), dec!(2.0)).unwrap()
            .with_madhab(Madhab::Shafi);
            
        let shafi_calc = BusinessZakatCalculator::new(assets.clone());
        let shafi_res = shafi_calc.with_hawl(true).calculate_zakat(&shafi_config).unwrap();
        
        assert!(!shafi_res.is_payable, "Shafi (Gold) should not be payable as 5000 < 8500");
        assert_eq!(shafi_res.nisab_threshold, dec!(8500.0));

        // 2. Test Hanafi (LowerOfTwo)
        let hanafi_config = ZakatConfig::new(dec!(100.0), dec!(2.0)).unwrap()
            .with_madhab(Madhab::Hanafi);
            
        let hanafi_calc = BusinessZakatCalculator::new(assets);
        let hanafi_res = hanafi_calc.with_hawl(true).calculate_zakat(&hanafi_config).unwrap();
        
        assert!(hanafi_res.is_payable, "Hanafi (LowerOfTwo) should be payable as 5000 > 1190");
        assert_eq!(hanafi_res.nisab_threshold, dec!(1190.0));
    }
}
