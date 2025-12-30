//! # Fiqh Compliance: Professional Income (Zakat al-Mustafad)
//!
//! ## Concept
//! - **Source**: Based on *Mal Mustafad* (wealth acquired during the year).
//! - **Modern Ijtihad**: Dr. Yusuf Al-Qaradawi (*Fiqh al-Zakah*) argues for immediate payment upon receipt, analogous to agriculture (Harvest Tax).
//!
//! ## Calculation Methods
//! - **Gross**: Pay immediately on total income (Stricter, similar to Ushr/Half-Ushr logic).
//! - **Net**: Deduct basic needs (*Hajah Asliyyah*) and debts before calculating surplus (Lenient).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::CalculateZakat;
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::config::ZakatConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IncomeCalculationMethod {
    #[default]
    Gross,
    Net,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct IncomeZakatCalculator {
    pub total_income: Decimal,
    pub basic_expenses: Decimal,
    pub method: IncomeCalculationMethod,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
}

impl IncomeZakatCalculator {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ..Default::default()
        }
    }

    pub fn income(mut self, income: impl IntoZakatDecimal) -> Self {
        if let Ok(i) = income.into_zakat_decimal() {
            self.total_income = i;
        }
        self
    }

    pub fn expenses(mut self, expenses: impl IntoZakatDecimal) -> Self {
        if let Ok(e) = expenses.into_zakat_decimal() {
            self.basic_expenses = e;
        }
        self
    }

    pub fn method(mut self, method: IncomeCalculationMethod) -> Self {
        self.method = method;
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

impl CalculateZakat for IncomeZakatCalculator {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        if self.total_income < Decimal::ZERO || self.basic_expenses < Decimal::ZERO {
            return Err(ZakatError::InvalidInput {
                field: "income_expenses".to_string(),
                value: "negative".to_string(),
                reason: "Income and expenses must be non-negative".to_string(),
                source_label: self.label.clone()
            });
        }

        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError {
                reason: "Gold price needed for Income Nisab".to_string(),
                source_label: self.label.clone()
            });
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError {
                reason: "Silver price needed for Income Nisab with current standard".to_string(),
                source_label: self.label.clone()
            });
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
                let combined_liabilities = ZakatDecimal::new(self.basic_expenses)
                    .safe_add(external_debt)?
                    .with_source(self.label.clone());
                (self.total_income, *combined_liabilities)
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
        let net_income = ZakatDecimal::new(total_assets)
            .safe_sub(liabilities)?
            .with_source(self.label.clone());
        trace.push(crate::types::CalculationStep::result("Net Zakatable Income", *net_income));
        
        trace.push(crate::types::CalculationStep::compare("Nisab Threshold", nisab_threshold_value));
        
        if *net_income >= nisab_threshold_value && *net_income > Decimal::ZERO {
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

    fn get_id(&self) -> uuid::Uuid {
        self.id
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
        
        let calc = IncomeZakatCalculator::new()
            .income(dec!(10000.0))
            .expenses(dec!(5000.0)) // Ignored in Gross
            .method(IncomeCalculationMethod::Gross);
        let res = calc.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250.0));
    }

    #[test]
    fn test_income_net() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Income 12,000. Expenses 4,000. Net 8,000.
        // Net < Nisab. Not Payable.
        
        let calc = IncomeZakatCalculator::new()
            .income(dec!(12000.0))
            .expenses(dec!(4000.0))
            .method(IncomeCalculationMethod::Net);
        let res = calc.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!res.is_payable);
        // (12000 - 4000) = 8000. 8000 < 8500.
    }
}
