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
use crate::types::{ZakatDetails, ZakatError, ErrorDetails};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};
use crate::validation::Validator;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display, strum::EnumString, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum IncomeCalculationMethod {
    #[default]
    Gross,
    Net,
}

impl crate::inputs::ToFfiString for IncomeCalculationMethod {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl crate::inputs::FromFfiString for IncomeCalculationMethod {
    type Err = strum::ParseError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
         use std::str::FromStr;
        Self::from_str(s)
    }
}

// MACRO USAGE
crate::zakat_ffi_export! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct IncomeZakatCalculator {
        pub income: Decimal,
        pub expenses: Decimal,
        pub method: IncomeCalculationMethod,
    }
}

impl Default for IncomeZakatCalculator {
    fn default() -> Self {
        let (liabilities_due_now, named_liabilities, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            income: Decimal::ZERO,
            expenses: Decimal::ZERO,
            method: IncomeCalculationMethod::default(),
            liabilities_due_now,
            named_liabilities,
            hawl_satisfied,
            label,
            id,
            acquisition_date,
            _input_errors,
        }
    }
}

impl IncomeZakatCalculator {
    // new() is provided by the macro

    pub fn from_amounts(
        income: impl IntoZakatDecimal,
        expenses: impl IntoZakatDecimal,
        debt: impl IntoZakatDecimal,
    ) -> Self {
        let mut calc = Self::default();
        calc = calc.income(income);
        calc = calc.expenses(expenses);
        calc = calc.debt(debt); // Uses macro's debt() setter -> liabilities_due_now
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

    /// Sets total income.
    pub fn income(mut self, income: impl IntoZakatDecimal) -> Self {
        match income.into_zakat_decimal() {
            Ok(v) => self.income = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    /// Sets basic living expenses.
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
}

impl CalculateZakat for IncomeZakatCalculator {
    fn validate_input(&self) -> Result<(), ZakatError> { self.validate() }
    fn get_label(&self) -> Option<String> { self.label.clone() }
    fn get_id(&self) -> uuid::Uuid { self.id }

    #[allow(deprecated)]
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.validate()?;
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        Validator::ensure_non_negative(&[
            ("income", self.income),
            ("expenses", self.expenses)
        ], self.label.clone())?;

        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                reason_key: "error-gold-price-required".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                reason_key: "error-silver-price-required".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        // Dynamic rate from strategy (default 2.5%)
        let rate = config.strategy.get_rules().trade_goods_rate;
        let external_debt = self.total_liabilities(); // Uses total of legacy + named

        // Collect any warnings
        let mut warnings = Vec::new();

        // Calculate total_assets and liabilities based on method
        let (total_assets, liabilities) = match self.method {
            IncomeCalculationMethod::Gross => {
                // Gross Method: 2.5% of Total Income.
                // Deducting debts is generally not standard in the Gross method (similar to agriculture),
                // but we deduct external_debt if provided to support flexible user requirements.
                
                // Warn user if expenses were set but will be ignored
                if self.expenses > Decimal::ZERO {
                    warnings.push("Expenses are ignored when using the Gross calculation method.".to_string());
                }
                
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

        // Override hawl_satisfied if acquisition_date is present
        let hawl_is_satisfied = if let Some(date) = self.acquisition_date {
            let tracker = crate::hawl::HawlTracker::new(chrono::Local::now().date_naive())
                .acquired_on(date);
            tracker.is_satisfied()
        } else {
            self.hawl_satisfied
        };

        let params = MonetaryCalcParams {
            total_assets,
            liabilities,
            nisab_threshold: nisab_threshold_value,
            rate,
            wealth_type: crate::types::WealthType::Income,
            label: self.label.clone(),
            hawl_satisfied: hawl_is_satisfied,
            trace_steps,
            warnings,
        };

        calculate_monetary_asset(params)
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
