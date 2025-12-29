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
    deductible_liabilities: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl IncomeZakatCalculator {
    pub fn new(
        total_income: impl Into<Decimal>,
        basic_expenses: impl Into<Decimal>,
        method: IncomeCalculationMethod,
        config: &ZakatConfig,
    ) -> Result<Self, ZakatError> {
        let income = total_income.into();
        let expenses = basic_expenses.into();

        if income < Decimal::ZERO || expenses < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Income and expenses must be non-negative".to_string()));
        }

        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::config::NisabStandard::Silver | crate::config::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Income Nisab".to_string()));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Income Nisab with current standard".to_string()));
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();
        
        Ok(Self {
            total_income: income,
            basic_expenses: expenses,
            method,
            nisab_threshold_value,
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

impl CalculateZakat for IncomeZakatCalculator {
    fn calculate_zakat(&self) -> Result<ZakatDetails, ZakatError> {
        // Income usually doesn't strictly require hawl if it's salary (paid upon receipt),
        // but if the user explicitly sets hawl_satisfied = false, we should respect it.
        if !self.hawl_satisfied {
             return Ok(ZakatDetails::not_payable(self.nisab_threshold_value, crate::types::WealthType::Income, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        let rate = dec!(0.025);
        let external_debt = self.deductible_liabilities;

        let (total_assets, liabilities) = match self.method {
            IncomeCalculationMethod::Gross => {
                // Gross Method: 2.5% of Total Income.
                // Deducting debts is generally not standard in the Gross method (similar to agriculture),
                // but we deduct external_debt if provided to support flexible user requirements.
                (self.total_income, external_debt)
            },
            IncomeCalculationMethod::Net => {
                // Net means (Income - Basic Living Expenses).
                // Then we also deduct any extra debts.
                (self.total_income, self.basic_expenses + external_debt)
            }
        };

        Ok(ZakatDetails::new(total_assets, liabilities, self.nisab_threshold_value, rate, crate::types::WealthType::Income)
            .with_label(self.label.clone().unwrap_or_default()))
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
        let res = calc.with_hawl(true).calculate_zakat().unwrap();
        
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
        let res = calc.with_hawl(true).calculate_zakat().unwrap();
        
        assert!(!res.is_payable);
        // (12000 - 4000) = 8000. 8000 < 8500.
    }
}
