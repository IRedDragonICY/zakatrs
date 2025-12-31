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
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::CalculateZakat;
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;
use crate::config::ZakatConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InvestmentType {
    #[default]
    Stock,
    Crypto,
    MutualFund,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentAssets {
    pub market_value: Decimal,
    pub investment_type: InvestmentType,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
}

impl InvestmentAssets {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ..Default::default()
        }
    }

    /// Creates a Stock investment asset with the specified market value.
    /// Defaults to Hawl satisfied.
    pub fn stock(value: impl IntoZakatDecimal) -> Self {
        Self::new()
            .value(value)
            .kind(InvestmentType::Stock)
            .hawl(true)
    }

    /// Creates a Crypto investment asset with the specified market value.
    /// Defaults to Hawl satisfied.
    pub fn crypto(value: impl IntoZakatDecimal) -> Self {
        Self::new()
            .value(value)
            .kind(InvestmentType::Crypto)
            .hawl(true)
    }

    pub fn value(mut self, value: impl IntoZakatDecimal) -> Self {
        if let Ok(v) = value.into_zakat_decimal() {
            self.market_value = v;
        }
        self
    }

    pub fn kind(mut self, kind: InvestmentType) -> Self {
        self.investment_type = kind;
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

impl CalculateZakat for InvestmentAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        if self.market_value < Decimal::ZERO {
            return Err(ZakatError::InvalidInput {
                field: "market_value".to_string(),
                value: "negative".to_string(),
                reason: "Market value must be non-negative".to_string(),
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
                reason: "Gold price needed for Investment Nisab".to_string(),
                source_label: self.label.clone()
            });
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError {
                reason: "Silver price needed for Investment Nisab with current standard".to_string(),
                source_label: self.label.clone()
            });
        }
        
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(nisab_threshold_value, crate::types::WealthType::Investment, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        // Requirement: 
        // Crypto: Treated as Trade Goods (2.5% if > Nisab).
        // Stocks: Market Value * 2.5% (Zakah on Principal + Profit).
        
        let total_assets = self.market_value;
        let liabilities = self.liabilities_due_now;
        let rate = dec!(0.025);

        // Build calculation trace
        let type_desc = match self.investment_type {
            InvestmentType::Stock => "Stocks",
            InvestmentType::Crypto => "Crypto",
            InvestmentType::MutualFund => "Mutual Fund",
        };

        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial(format!("Market Value ({})", type_desc), total_assets));
        trace.push(crate::types::CalculationStep::subtract("Debts Due Now", liabilities));
        
        let net_assets = ZakatDecimal::new(total_assets)
            .safe_sub(liabilities)?
            .with_source(self.label.clone());
        trace.push(crate::types::CalculationStep::result("Net Investment Assets", *net_assets));
        trace.push(crate::types::CalculationStep::compare("Nisab Threshold", nisab_threshold_value));
        
        if *net_assets >= nisab_threshold_value && *net_assets > Decimal::ZERO {
            trace.push(crate::types::CalculationStep::rate("Applied Rate (2.5%)", rate));
        } else {
             trace.push(crate::types::CalculationStep::info("Net Assets below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails::with_trace(total_assets, liabilities, nisab_threshold_value, rate, crate::types::WealthType::Investment, trace)
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
