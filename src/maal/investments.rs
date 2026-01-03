//! # Fiqh Compliance: Stocks & Investments
//!
//! ## Classification
//! - **Stocks/Crypto**: Classified as *Urud al-Tijarah* (Trade Goods) when held for capital appreciation.
//! - **Standard**: Subject to 2.5% Zakat on Market Value if Nisab is reached.
//!
//! ## Sources
//! - **AAOIFI Sharia Standard No. 35**: Specifies that shares acquired for trading are Zakatable at market value.
//! - **IIFA Resolutions**: Cryptocurrencies recognized as wealth (*Mal*) are subject to Zakat if they meet conditions of value and possession.

use rust_decimal::Decimal;
use crate::types::{ZakatDetails, ZakatError, ErrorDetails, InvalidInputDetails};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display, strum::EnumString)]
pub enum InvestmentType {
    #[default]
    Stock,
    Crypto,
    MutualFund,
}

impl crate::inputs::ToFfiString for InvestmentType {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl crate::inputs::FromFfiString for InvestmentType {
    type Err = strum::ParseError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
         use std::str::FromStr;
        Self::from_str(s)
    }
}

// MACRO USAGE
crate::zakat_ffi_export! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InvestmentAssets {
        pub value: Decimal,
        pub investment_type: InvestmentType,
        /// Purification rate (e.g., 0.05 for 5%) to cleanse non-halal income.
        pub purification_rate: Option<Decimal>,
    }
}

impl Default for InvestmentAssets {
    fn default() -> Self {
        let (liabilities_due_now, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            value: Decimal::ZERO,
            investment_type: InvestmentType::default(),
            purification_rate: None,
            liabilities_due_now,
            hawl_satisfied,
            label,
            id,
            acquisition_date,
            _input_errors,
        }
    }
}

impl InvestmentAssets {
    // new() is provided by the macro

    pub fn stock(value: impl IntoZakatDecimal) -> Self {
        Self::default().value(value).kind(InvestmentType::Stock)
    }

    pub fn crypto(value: impl IntoZakatDecimal) -> Self {
        Self::default().value(value).kind(InvestmentType::Crypto)
    }

    pub fn value(mut self, value: impl IntoZakatDecimal) -> Self {
        match value.into_zakat_decimal() {
            Ok(v) => self.value = v,
            Err(e) => self._input_errors.push(e),
        }
        self
    }

    pub fn kind(mut self, kind: InvestmentType) -> Self {
        self.investment_type = kind;
        self
    }

    /// Sets the purification rate (Tathir) to cleanse non-halal income.
    /// Example: `0.05` for 5% legacy income deduction.
    pub fn purify(mut self, rate: impl IntoZakatDecimal) -> Self {
        match rate.into_zakat_decimal() {
            Ok(v) => self.purification_rate = Some(v),
            Err(e) => self._input_errors.push(e),
        }
        self
    }
}

impl CalculateZakat for InvestmentAssets {
    fn validate_input(&self) -> Result<(), ZakatError> { self.validate() }
    fn get_label(&self) -> Option<String> { self.label.clone() }
    fn get_id(&self) -> uuid::Uuid { self.id }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.validate()?;
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        // Specific input validation
        if self.value < Decimal::ZERO {
             return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "market_value".to_string(),
                value: "negative".to_string(),
                reason_key: "error-negative-value".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }
        if self.liabilities_due_now < Decimal::ZERO {
             return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "debt".to_string(),
                value: "negative".to_string(),
                reason_key: "error-negative-value".to_string(),
                args: None,
                source_label: self.label.clone(),
                asset_id: None,
            })));
        }

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

        // Requirement: 
        // Crypto: Treated as Trade Goods (2.5% if > Nisab).
        // Stocks: Market Value * 2.5% (Zakah on Principal + Profit).
        
        // Dynamic rate from strategy (default 2.5%)
        let rate = config.strategy.get_rules().trade_goods_rate;

        // Build calculation trace
        let type_desc = match self.investment_type {
            InvestmentType::Stock => "Stocks",
            InvestmentType::Crypto => "Crypto",
            InvestmentType::MutualFund => "Mutual Fund",
        };

        let mut trace_steps = vec![
            crate::types::CalculationStep::initial("step-market-value", format!("Market Value ({})", type_desc), self.value)
                 .with_args(std::collections::HashMap::from([("type".to_string(), type_desc.to_string())]))
        ];

        // Apply Purification if set
        let zakatable_gross = if let Some(purify_rate) = self.purification_rate {
             let impure_amount = ZakatDecimal::new(self.value)
                .safe_mul(purify_rate)?
                .with_source(self.label.clone());
             
             trace_steps.push(crate::types::CalculationStep::rate("step-purification-rate", "Purification Rate (Tathir)", purify_rate));
             trace_steps.push(crate::types::CalculationStep::subtract("step-purification-amount", "Impure Amount Deducted", *impure_amount));
             
             let puri_val = ZakatDecimal::new(self.value)
                .safe_sub(*impure_amount)?
                .with_source(self.label.clone());
             
             trace_steps.push(crate::types::CalculationStep::result("step-purified-value", "Purified Gross Value", *puri_val));
             *puri_val
        } else {
            self.value
        };

        // Override hawl_satisfied if acquisition_date is present
        let hawl_is_satisfied = if let Some(date) = self.acquisition_date {
            let tracker = crate::hawl::HawlTracker::new(chrono::Local::now().date_naive())
                .acquired_on(date);
            tracker.is_satisfied()
        } else {
            self.hawl_satisfied
        };  

        let params = MonetaryCalcParams {
            total_assets: zakatable_gross,
            liabilities: self.liabilities_due_now, // Uses macro field
            nisab_threshold: nisab_threshold_value,
            rate,
            wealth_type: crate::types::WealthType::Investment,
            label: self.label.clone(),
            hawl_satisfied: hawl_is_satisfied,
            trace_steps,
            warnings: Vec::new(),
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
    fn test_crypto_investment() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100), ..Default::default() };
        // Nisab 8500.
        // Crypto worth 10,000.
        // Due 250.
        
        let inv = InvestmentAssets::new()
            .value(10000.0)
            .kind(InvestmentType::Crypto);
            
        let res = inv.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250));
    }
}
