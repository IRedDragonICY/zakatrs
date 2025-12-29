use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

pub enum InvestmentType {
    Stock,
    Crypto,
    MutualFund,
}

pub struct InvestmentAssets {
    pub market_value: Decimal,
    pub investment_type: InvestmentType,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl InvestmentAssets {
    pub fn new(
        market_value: impl IntoZakatDecimal,
        investment_type: InvestmentType,
    ) -> Result<Self, ZakatError> {
        let value = market_value.into_zakat_decimal()?;

        if value < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Market value must be non-negative".to_string()));
        }

        Ok(Self {
            market_value: value,
            investment_type,
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

impl CalculateZakat for InvestmentAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Investment Nisab".to_string()));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Investment Nisab with current standard".to_string()));
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

        Ok(ZakatDetails::new(total_assets, liabilities, nisab_threshold_value, rate, crate::types::WealthType::Investment)
            .with_label(self.label.clone().unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_investment() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // Nisab 8500.
        // Crypto worth 10,000.
        // Due 250.
        
        let inv = InvestmentAssets::new(dec!(10000.0), InvestmentType::Crypto).unwrap();
        let res = inv.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250.0));
    }
}
