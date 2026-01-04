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
use crate::types::{ZakatDetails, ZakatError, ErrorDetails};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};
use crate::validation::Validator;


/// Determines the Zakat calculation logic based on the investor's intention (Niyyah).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display, strum::EnumString, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InvestmentStrategy {
    /// Held for short-term trading or capital growth. 
    /// Zakat is due on the full market value as they are treated as trade goods.
    #[default]
    CapitalAppreciation,
    /// Held for long-term passive income or retirement. 
    /// Uses the 30% proxy rule (Deduction of fixed assets). Instead of analyzing 
    /// balance sheets for every stock, 30% of market value is taken as the 
    /// zakatable portion (representing liquid assets like cash and receivables).
    DividendYield,
}

impl crate::inputs::ToFfiString for InvestmentStrategy {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl crate::inputs::FromFfiString for InvestmentStrategy {
    type Err = strum::ParseError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
         use std::str::FromStr;
        Self::from_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display, strum::EnumString, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
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
    /// Represents investment assets (Stocks, Crypto, Mutual Funds) with strategy-based valuation.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct InvestmentAssets {
        /// Current market value of the total holding.
        pub value: Decimal,
        /// Type of investment (affects classification but usually not the rate).
        pub investment_type: InvestmentType,
        /// Purification rate (e.g., 0.05 for 5%) to cleanse non-halal income (Tathir).
        pub purification_rate: Option<Decimal>,
        /// Zakat strategy based on intention (Niyyah). 
        /// Differentiates between trading (100% base) and long-term holding (30% proxy).
        #[serde(default)]
        pub strategy: InvestmentStrategy,
    }
}

#[allow(deprecated)] // Uses deprecated `liabilities_due_now` for backward compat
impl Default for InvestmentAssets {
    fn default() -> Self {
        let (liabilities_due_now, named_liabilities, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            value: Decimal::ZERO,
            investment_type: InvestmentType::default(),
            purification_rate: None,
            strategy: Default::default(),
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

    /// Sets the investment strategy (Niyyah).
    pub fn strategy(mut self, strategy: InvestmentStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

impl CalculateZakat for InvestmentAssets {
    fn validate_input(&self) -> Result<(), ZakatError> { self.validate() }
    fn get_label(&self) -> Option<String> { self.label.clone() }
    fn get_id(&self) -> uuid::Uuid { self.id }

    #[allow(deprecated)]
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.validate()?;
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        // Specific input validation
        Validator::ensure_non_negative(&[
            ("market_value", self.value),
            ("debt", self.liabilities_due_now)
        ], self.label.clone())?;

        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-gold-price-required".to_string(),
                source_label: self.label.clone(),
                suggestion: Some("Run with --gold-price X or set ZAKAT_GOLD_PRICE env var.".to_string()),
                ..Default::default()
            })));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-silver-price-required".to_string(),
                source_label: self.label.clone(),
                suggestion: Some("Run with --silver-price X or set ZAKAT_SILVER_PRICE env var.".to_string()),
                ..Default::default()
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

        // START CHANGE: Feature 3 (Investment Strategy)
        let zakatable_base = match self.strategy {
            InvestmentStrategy::CapitalAppreciation => self.value,
            InvestmentStrategy::DividendYield => {
                 use rust_decimal_macros::dec;
                 // 30% Proxy Rule for "Net Zakatable Assets"
                 let proxy_rate = dec!(0.30);
                 let zakatable_portion = self.value * proxy_rate;
                 
                 trace_steps.push(crate::types::CalculationStep::rate(
                     "step-dividend-proxy", 
                     "Held for Dividends: 30% Proxy Rule Applied", 
                     proxy_rate
                 ));
                 trace_steps.push(crate::types::CalculationStep::result(
                     "step-zakatable-portion", 
                     "Net Zakatable Assets (Proxy)", 
                     zakatable_portion
                 ));
                 
                 zakatable_portion
            }
        };
        // END CHANGE

        // Apply Purification if set
        let zakatable_gross = if let Some(purify_rate) = self.purification_rate {
             // Purify calculate on the BASE (zakatable portion)
             // Purification is typically on dividends/income, but often applied to total value 
             // to "cleanse" the holding. 
             // If Strategy is DividendYield, we already reduced to 30%.
             // Purification should arguably apply to the *Dividend* not the Asset Value, 
             // BUT `purification_rate` here is often interpreted as "Portfolio purification".
             // Let's apply it to the `zakatable_base`.
             
             let impure_amount = ZakatDecimal::new(zakatable_base)
                .safe_mul(purify_rate)?
                .with_source(self.label.clone());
             
             trace_steps.push(crate::types::CalculationStep::rate("step-purification-rate", "Purification Rate (Tathir)", purify_rate));
             trace_steps.push(crate::types::CalculationStep::subtract("step-purification-amount", "Impure Amount Deducted", *impure_amount));
             
             let puri_val = ZakatDecimal::new(zakatable_base)
                .safe_sub(*impure_amount)?
                .with_source(self.label.clone());
             
             trace_steps.push(crate::types::CalculationStep::result("step-purified-value", "Purified Gross Value", *puri_val));
             *puri_val
        } else {
            zakatable_base
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
            liabilities: self.total_liabilities(), // Uses total of legacy + named
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

    #[test]
    fn test_investment_strategy_dividend_yield() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100), ..Default::default() };
        // Value 100,000. 
        // Strategy: DividendYield -> Proxy 30% = 30,000.
        // Zakat 2.5% of 30,000 = 750.
        
        let inv = InvestmentAssets::new()
            .value(100000.0)
            .strategy(InvestmentStrategy::DividendYield)
            .hawl(true);
            
        let res = inv.calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(750));
        // Verify trace contains proxy message
        let trace = res.calculation_trace.0;
        assert!(trace.iter().any(|s| s.description.contains("30% Proxy")));
    }
}
