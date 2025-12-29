use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub enum InvestmentType {
    Stock,
    Crypto,
    MutualFund,
}

pub struct InvestmentAssets {
    pub market_value: Decimal,
    pub investment_type: InvestmentType,
    pub nisab_threshold_value: Decimal,
}

impl InvestmentAssets {
    pub fn new(
        market_value: impl Into<Decimal>,
        investment_type: InvestmentType,
        config: &ZakatConfig,
    ) -> Result<Self, ZakatError> {
         if config.gold_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Gold price needed for Investment Nisab".to_string()));
        }
        
        let nisab_threshold_value = config.gold_price_per_gram * config.get_nisab_gold_grams();
        
        Ok(Self {
            market_value: market_value.into(),
            investment_type,
            nisab_threshold_value,
        })
    }
}

impl CalculateZakat for InvestmentAssets {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        // Requirement: 
        // Crypto: Treated as Trade Goods (2.5% if > Nisab).
        // Stocks: Market Value * 2.5% (MVP).
        
        let total_assets = self.market_value;
        let liabilities = extra_debts.unwrap_or(Decimal::ZERO);
        let rate = dec!(0.025);

        Ok(ZakatDetails::new(total_assets, liabilities, self.nisab_threshold_value, rate, crate::types::WealthType::Investment))
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
        
        let inv = InvestmentAssets::new(dec!(10000.0), InvestmentType::Crypto, &config).unwrap();
        let res = inv.calculate_zakat(None).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(250.0));
    }
}
