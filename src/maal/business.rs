use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub struct BusinessAssets {
    pub cash_on_hand: Decimal,
    pub inventory_value: Decimal,
    pub receivables: Decimal,
    pub short_term_liabilities: Decimal,
}

impl BusinessAssets {
    pub fn new(
        cash: Decimal,
        inventory: Decimal,
        receivables: Decimal,
        short_term_liabilities: Decimal,
    ) -> Self {
        Self {
            cash_on_hand: cash,
            inventory_value: inventory,
            receivables,
            short_term_liabilities,
        }
    }
}

impl CalculateZakat for BusinessAssets {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        // Zakat Trade = (Cash + Inventory + Receivables) - Short Term Debt
        // Note: extra_debts passed here would be arguably redundant if short_term_liabilities covers it,
        // but we treat short_term_liabilities as business operational debt, and extra_debts as potentially personal debt if applicable,
        // or we can sum them. Let's sum them to be safe/flexible.
        
        // Ensure values are not negative (basic sanitation)
        if self.cash_on_hand < Decimal::ZERO || self.inventory_value < Decimal::ZERO || self.receivables < Decimal::ZERO {
             return Err(ZakatError::InvalidInput("Assets cannot be negative".to_string()));
        }

        // Nisab is 85g Gold
        // We need a way to get the gold price.
        // The trait calculate_zakat doesn't take config.
        // Design flaw in my hasty trait definition? 
        // Solved by refactoring: The Struct should hold the config or the Nisab value directly needed.
        // Let's assume the caller configures the BusinessAssets struct WITH the nisab threshold or config,
        // OR we change the trait (breaking change).
        // Since I can't easily change the trait signature across all files without big diffs, 
        // I'll assume the struct must be initialized with the Nisab threshold.
        // Wait, I can't pass config to calculate_zakat.
        // I will adhere to the trait: fn calculate_zakat(&self, debts: Option<Decimal>)
        // So BusinessAssets needs to know the Nisab value upon creation.
        
        // BUT wait, I haven't implemented the trait in a way that requires config in the struct yet for others except PreciousMetal which took it in 'new'.
        // So I will replicate that pattern: Pass config to `new` and store the nisab threshold.
        // Actually, storing the Threshold is better than storing the Config, decoupling it.
        Err(ZakatError::ConfigurationError("Please use BusinessZakatCalculator wrapper or similar".to_string()))
    }
}

// Better approach to avoid storing state:
// The Trait is fine, but the struct needs the context.
pub struct BusinessZakatCalculator {
    assets: BusinessAssets,
    nisab_threshold_value: Decimal,
}

impl BusinessZakatCalculator {
    pub fn new(assets: BusinessAssets, config: &ZakatConfig) -> Result<Self, ZakatError> {
        if config.gold_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Gold price needed for Business Nisab".to_string()));
        }
        
        // Nisab for Business is 85g Gold equivalent (or Silver, but standard is Gold).
        let nisab_threshold_value = config.gold_price_per_gram * config.get_nisab_gold_grams();
        
        Ok(Self {
            assets,
            nisab_threshold_value,
        })
    }
}

impl CalculateZakat for BusinessZakatCalculator {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        let gross_assets = self.assets.cash_on_hand + self.assets.inventory_value + self.assets.receivables;
        let business_debt = self.assets.short_term_liabilities;
        
        let total_assets = gross_assets;
        
        // Logic: (Assets - Liabilities) >= Nisab
        // ZakatDetails expects:
        // total_assets, deductible_liabilities, nisab values
        
        let other_debt = extra_debts.unwrap_or(Decimal::ZERO);
        let total_liabilities = business_debt + other_debt;

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
        let result = calculator.calculate_zakat(None).unwrap();

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
         let result = calculator.calculate_zakat(None).unwrap();
         
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
        let result = calculator.calculate_zakat(None).unwrap();
        
        assert!(!result.is_payable);
        assert_eq!(result.net_assets, dec!(80000000.0));
    }
}
