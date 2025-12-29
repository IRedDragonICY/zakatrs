use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub enum IncomeCalculationMethod {
    Gross,
    Net,
}

pub struct IncomeZakatCalculator {
    total_income: Decimal,
    basic_expenses: Decimal,
    method: IncomeCalculationMethod,
    nisab_threshold_value: Decimal,
}

impl IncomeZakatCalculator {
    pub fn new(
        total_income: impl Into<Decimal>,
        basic_expenses: impl Into<Decimal>,
        method: IncomeCalculationMethod,
        config: &ZakatConfig,
    ) -> Result<Self, ZakatError> {
         if config.gold_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Gold price needed for Income Nisab".to_string()));
        }
        
        let nisab_threshold_value = config.gold_price_per_gram * config.get_nisab_gold_grams();
        
        Ok(Self {
            total_income: total_income.into(),
            basic_expenses: basic_expenses.into(),
            method,
            nisab_threshold_value,
        })
    }
}

impl CalculateZakat for IncomeZakatCalculator {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        let rate = dec!(0.025);
        let external_debt = extra_debts.unwrap_or(Decimal::ZERO);

        let (total_assets, liabilities) = match self.method {
            IncomeCalculationMethod::Gross => {
                // Gross means we take 2.5% of the Total Income directly. 
                // Any debts passed in would technically reduce the "Wealth" context if we treat it as "Assets",
                // but strictly speaking Gross method usually ignores expenses.
                // However, to keep it consistent with ZakatDetails structure:
                // We'll treat total_income as assets, and if the user supplied extra_debts, we deduct them 
                // ONLY if logical. In Gross method, usually no deductions.
                // But let's support debt deduction if explicitly passed via CalculateZakat.
                (self.total_income, external_debt)
            },
            IncomeCalculationMethod::Net => {
                // Net means (Income - Basic Living Expenses).
                // Then we also deduct any extra debts.
                (self.total_income, self.basic_expenses + external_debt)
            }
        };

        Ok(ZakatDetails::new(total_assets, liabilities, self.nisab_threshold_value, rate, crate::types::WealthType::Income))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_income_gross() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Income 10,000. Gross.
        // Due 250.
        
        let calc = IncomeZakatCalculator::new(dec!(10000.0), dec!(5000.0), IncomeCalculationMethod::Gross, &config).unwrap();
        let res = calc.calculate_zakat(None).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250.0));
    }

    #[test]
    fn test_income_net() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Income 12,000. Expenses 4,000. Net 8,000.
        // Net < Nisab. Not Payable.
        
        let calc = IncomeZakatCalculator::new(dec!(12000.0), dec!(4000.0), IncomeCalculationMethod::Net, &config).unwrap();
        let res = calc.calculate_zakat(None).unwrap();
        
        assert!(!res.is_payable);
        // (12000 - 4000) = 8000. 8000 < 8500.
    }
}
