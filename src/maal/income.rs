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
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum IncomeCalculationMethod {
    #[default]
    Gross,
    Net,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeZakatCalculator {
    pub income: Decimal,
    pub expenses: Decimal,
    pub debt: Decimal,
    pub method: IncomeCalculationMethod,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
    // Hidden field for deferred input validation errors
    #[serde(skip)]
    _input_errors: Vec<ZakatError>,
}

impl Default for IncomeZakatCalculator {
    fn default() -> Self {
        Self {
            income: Decimal::ZERO,
            expenses: Decimal::ZERO,
            debt: Decimal::ZERO,
            method: IncomeCalculationMethod::default(),
            hawl_satisfied: false,
            label: None,
            id: uuid::Uuid::new_v4(),
            _input_errors: Vec::new(),
        }
    }
}

impl IncomeZakatCalculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_amounts(
        income: impl IntoZakatDecimal,
        expenses: impl IntoZakatDecimal,
        debt: impl IntoZakatDecimal,
    ) -> Self {
        let mut calc = Self::default();
        calc = calc.income(income);
        calc = calc.expenses(expenses);
        calc = calc.debt(debt);
        calc
    }

    /// Creates an Income Zakat calculator for a salary amount.
    /// Defaults to Gross calculation method and Hawl satisfied (immediate payment).
    pub fn from_salary(amount: impl IntoZakatDecimal) -> Self {
        Self::new()
            .income(amount)
            .method(IncomeCalculationMethod::Gross)
            .hawl(true)
    }

    pub fn validate(&self) -> Result<(), ZakatError> {
        match self._input_errors.len() {
            0 => Ok(()),
            1 => Err(self._input_errors[0].clone()),
            _ => Err(ZakatError::MultipleErrors(self._input_errors.clone())),
        }
    }

    /// Sets total income.
    ///
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn income(mut self, income: impl IntoZakatDecimal) -> Self {
        match income.into_zakat_decimal() {
            Ok(v) => self.income = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    /// Sets basic living expenses.
    ///
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn expenses(mut self, expenses: impl IntoZakatDecimal) -> Self {
        match expenses.into_zakat_decimal() {
            Ok(v) => self.expenses = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    pub fn method(mut self, method: IncomeCalculationMethod) -> Self {
        self.method = method;
        self
    }

    /// Sets deductible debt.
    ///
    /// # Panics
    /// Panics if the value cannot be converted to a valid decimal.
    pub fn debt(mut self, debt: impl IntoZakatDecimal) -> Self {
        match debt.into_zakat_decimal() {
            Ok(v) => self.debt = v,
            Err(e) => self._input_errors.push(e),
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

    /// Restores the asset ID (for database/serialization restoration).
    pub fn with_id(mut self, id: uuid::Uuid) -> Self {
        self.id = id;
        self
    }
}

impl CalculateZakat for IncomeZakatCalculator {
    fn validate_input(&self) -> Result<(), ZakatError> {
        self.validate()
    }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.validate()?;
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        if self.income < Decimal::ZERO || self.expenses < Decimal::ZERO {
            return Err(ZakatError::InvalidInput {
                field: "income_expenses".to_string(),
                value: "negative".to_string(),
                reason: "Income and expenses must be non-negative".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
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
                source_label: self.label.clone(),
                asset_id: None,
            });
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError {
                reason: "Silver price needed for Income Nisab with current standard".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            });
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        // Dynamic rate from strategy (default 2.5%)
        let rate = config.strategy.get_rules().trade_goods_rate;
        let external_debt = self.debt;

        // Calculate total_assets and liabilities based on method
        let (total_assets, liabilities) = match self.method {
            IncomeCalculationMethod::Gross => {
                // Gross Method: 2.5% of Total Income.
                // Deducting debts is generally not standard in the Gross method (similar to agriculture),
                // but we deduct external_debt if provided to support flexible user requirements.
                (self.income, external_debt)
            },
            IncomeCalculationMethod::Net => {
                // Net means (Income - Basic Living Expenses).
                // Then we also deduct any extra debts.
                let combined_liabilities = ZakatDecimal::new(self.expenses)
                    .safe_add(external_debt)?
                    .with_source(self.label.clone());
                (self.income, *combined_liabilities)
            }
        };

        // Build trace steps based on method
        let mut trace_steps = vec![
            crate::types::CalculationStep::initial("step-total-income", "Total Income", self.income),
        ];
        
        match self.method {
            IncomeCalculationMethod::Net => {
                trace_steps.push(crate::types::CalculationStep::subtract("step-basic-expenses", "Basic Living Expenses", self.expenses));
            }
            IncomeCalculationMethod::Gross => {
                trace_steps.push(crate::types::CalculationStep::info("info-gross-method", "Gross Method used (Expenses not deducted)"));
            }
        }

        let params = MonetaryCalcParams {
            total_assets,
            liabilities,
            nisab_threshold: nisab_threshold_value,
            rate,
            wealth_type: crate::types::WealthType::Income,
            label: self.label.clone(),
            hawl_satisfied: self.hawl_satisfied,
            trace_steps,
        };

        calculate_monetary_asset(params)
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
    use rust_decimal_macros::dec;

    #[test]
    fn test_income_gross() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100), ..Default::default() };
        // Nisab 8500.
        // Income 10,000. Gross.
        // Due 250.
        
        let calc = IncomeZakatCalculator::new()
            .income(10000.0)
            .expenses(5000.0) // Ignored in Gross
            .method(IncomeCalculationMethod::Gross);
        let res = calc.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250));
    }

    #[test]
    fn test_income_net() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100), ..Default::default() };
        // Nisab 8500.
        // Income 12,000. Expenses 4,000. Net 8,000.
        // Net < Nisab. Not Payable.
        
        let calc = IncomeZakatCalculator::new()
            .income(12000.0)
            .expenses(4000.0)
            .method(IncomeCalculationMethod::Net);
        let res = calc.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!res.is_payable);
        // (12000 - 4000) = 8000. 8000 < 8500.
    }
}
