use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

pub enum IncomeCalculationMethod {
    Gross,
    Net,
}

pub struct IncomeZakatCalculator {
    total_income: Decimal,
    basic_expenses: Decimal,
    method: IncomeCalculationMethod,
    liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl IncomeZakatCalculator {
    pub fn new(
        total_income: impl IntoZakatDecimal,
        basic_expenses: impl IntoZakatDecimal,
        method: IncomeCalculationMethod,
    ) -> Result<Self, ZakatError> {
        let income = total_income.into_zakat_decimal()?;
        let expenses = basic_expenses.into_zakat_decimal()?;

        if income < Decimal::ZERO || expenses < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Income and expenses must be non-negative".to_string(), None));
        }

        Ok(Self {
            total_income: income,
            basic_expenses: expenses,
            method,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        })
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

impl CalculateZakat for IncomeZakatCalculator {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Income Nisab".to_string(), None));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Income Nisab with current standard".to_string(), None));
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        // Income usually doesn't strictly require hawl if it's salary (paid upon receipt),
        // but if the user explicitly sets hawl_satisfied = false, we should respect it.
        if !self.hawl_satisfied {
             return Ok(ZakatDetails::below_threshold(nisab_threshold_value, crate::types::WealthType::Income, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        let rate = dec!(0.025);
        let external_debt = self.liabilities_due_now;

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
                let combined_liabilities = self.basic_expenses
                    .checked_add(external_debt)
                    .ok_or(ZakatError::CalculationError("Overflow summing income liabilities".to_string(), None))?;
                (self.total_income, combined_liabilities)
            }
        };

        // Build calculation trace
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial("Total Income", self.total_income));
        
        match self.method {
            IncomeCalculationMethod::Net => {
                trace.push(crate::types::CalculationStep::subtract("Basic Living Expenses", self.basic_expenses));
            }
            IncomeCalculationMethod::Gross => {
                trace.push(crate::types::CalculationStep::info("Gross Method used (Expenses not deducted)"));
            }
        }

        trace.push(crate::types::CalculationStep::subtract("Debts Due Now", external_debt));
        let net_income = total_assets - liabilities;
        trace.push(crate::types::CalculationStep::result("Net Zakatable Income", net_income));
        
        trace.push(crate::types::CalculationStep::compare("Nisab Threshold", nisab_threshold_value));
        
        if net_income >= nisab_threshold_value && net_income > Decimal::ZERO {
            trace.push(crate::types::CalculationStep::rate("Applied Rate (2.5%)", rate));
        } else {
            trace.push(crate::types::CalculationStep::info("Net Income below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails::with_trace(total_assets, liabilities, nisab_threshold_value, rate, crate::types::WealthType::Income, trace)
            .with_label(self.label.clone().unwrap_or_default()))
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
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
        
        let calc = IncomeZakatCalculator::new(dec!(10000.0), dec!(5000.0), IncomeCalculationMethod::Gross).unwrap();
        let res = calc.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250.0));
    }

    #[test]
    fn test_income_net() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Income 12,000. Expenses 4,000. Net 8,000.
        // Net < Nisab. Not Payable.
        
        let calc = IncomeZakatCalculator::new(dec!(12000.0), dec!(4000.0), IncomeCalculationMethod::Net).unwrap();
        let res = calc.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!res.is_payable);
        // (12000 - 4000) = 8000. 8000 < 8500.
    }
}
