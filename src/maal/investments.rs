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
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InvestmentType {
    #[default]
    Stock,
    Crypto,
    MutualFund,
}

// MACRO USAGE
crate::zakat_asset! {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InvestmentAssets {
        pub value: Decimal,
        pub investment_type: InvestmentType,
    }
}

impl Default for InvestmentAssets {
    fn default() -> Self {
        let (liabilities_due_now, hawl_satisfied, label, id, _input_errors) = Self::default_common();
        Self {
            value: Decimal::ZERO,
            investment_type: InvestmentType::default(),
            liabilities_due_now,
            hawl_satisfied,
            label,
            id,
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
             return Err(ZakatError::InvalidInput {
                field: "market_value".to_string(),
                value: "negative".to_string(),
                reason: "Market value must be non-negative".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            });
        }
        if self.liabilities_due_now < Decimal::ZERO {
             return Err(ZakatError::InvalidInput {
                field: "debt".to_string(),
                value: "negative".to_string(),
                reason: "Debt must be non-negative".to_string(),
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
                reason: "Gold price needed for Investment Nisab".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            });
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError {
                reason: "Silver price needed for Investment Nisab with current standard".to_string(),
                source_label: self.label.clone(),
                asset_id: None,
            });
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

        let trace_steps = vec![
            crate::types::CalculationStep::initial("step-market-value", format!("Market Value ({})", type_desc), self.value)
                 .with_args(std::collections::HashMap::from([("type".to_string(), type_desc.to_string())]))
        ];

        let params = MonetaryCalcParams {
            total_assets: self.value,
            liabilities: self.liabilities_due_now, // Uses macro field
            nisab_threshold: nisab_threshold_value,
            rate,
            wealth_type: crate::types::WealthType::Investment,
            label: self.label.clone(),
            hawl_satisfied: self.hawl_satisfied,
            trace_steps,
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
