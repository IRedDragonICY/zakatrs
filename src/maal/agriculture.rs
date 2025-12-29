use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

pub enum IrrigationMethod {
    Rain, // Natural, 10%
    Irrigated, // Artificial/Costly, 5%
    Mixed, // Both, 7.5%
}

pub struct AgricultureAssets {
    pub harvest_weight_kg: Decimal,
    pub price_per_kg: Decimal,
    pub irrigation: IrrigationMethod,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl AgricultureAssets {
    pub fn new(
        harvest_weight_kg: impl IntoZakatDecimal,
        price_per_kg: impl IntoZakatDecimal,
        irrigation: IrrigationMethod,
    ) -> Result<Self, ZakatError> {
        let weight = harvest_weight_kg.into_zakat_decimal()?;
        let price = price_per_kg.into_zakat_decimal()?;

        if weight < Decimal::ZERO || price < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Harvest weight and price must be non-negative".to_string()));
        }

        Ok(Self {
            harvest_weight_kg: weight,
            price_per_kg: price,
            irrigation,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        })
    }

    /// Creates a new AgricultureAssets instance from Wasaq units.
    /// 1 Wasaq is approximately 130.6 kg.
    pub fn from_wasaq(
        wasaq: impl IntoZakatDecimal,
        price_per_kg: impl IntoZakatDecimal,
        irrigation: IrrigationMethod,
    ) -> Result<Self, ZakatError> {
        let wasaq_value = wasaq.into_zakat_decimal()?;
        let wasaq_in_kg = dec!(130.6);
        let weight_kg = wasaq_value * wasaq_in_kg;
        Self::new(weight_kg, price_per_kg, irrigation)
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

impl CalculateZakat for AgricultureAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        let rate = match self.irrigation {
            IrrigationMethod::Rain => dec!(0.10),
            IrrigationMethod::Irrigated => dec!(0.05),
            IrrigationMethod::Mixed => dec!(0.075),
        };
        
        let nisab_threshold_kg = config.get_nisab_agriculture_kg();

        let total_value = self.harvest_weight_kg * self.price_per_kg;
        let nisab_value = nisab_threshold_kg * self.price_per_kg; 
        
        let liabilities = self.liabilities_due_now;
        
        // ZakatDetails logic overrides 'is_payable' based on net_assets usually, but here Nisab is on Quantity.
        // We need to be careful. If Quantity > Nisab, then we pay rate on Total.
        // If we strictly follow "Net Assets" logic, we would deduct debt from value first.
        // But for Agriculture, usually checks yield quantity first.
        // Let's ensure ZakatDetails reflects the state correctly.
        
        let net_value = total_value - liabilities; // Just for reporting
        
        let zakat_due = if net_value >= nisab_value {
             net_value * rate
        } else {
             Decimal::ZERO
        };

        let is_payable = zakat_due > Decimal::ZERO;

        Ok(ZakatDetails {
            total_assets: total_value,
            liabilities_due_now: liabilities,
            net_assets: net_value,
            nisab_threshold: nisab_value,
            is_payable,
            zakat_due,
            wealth_type: crate::types::WealthType::Agriculture,
            status_reason: None,
            label: self.label.clone(),
            payload: crate::types::PaymentPayload::Monetary(zakat_due),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agriculture_rain() {
        let config = ZakatConfig::default(); // default 653kg
        // 1000 > 653. 
        // Rain -> 10%.
        // Price 1.0 -> Value 1000.
        // Due 100.
        
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Rain).unwrap();
        let res = agri.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(100.0));
    }

    #[test]
    fn test_agriculture_irrigated() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Irrigated).unwrap();
        let res = agri.with_hawl(true).calculate_zakat(&config).unwrap();
        
        // Irrigated -> 5%.
        // Due 50.
        assert_eq!(res.zakat_due, dec!(50.0));
    }
    
    #[test]
    fn test_agriculture_mixed() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Mixed).unwrap();
        let res = agri.with_hawl(true).calculate_zakat(&config).unwrap();
        
        // Mixed -> 7.5%.
        // Due 75.
        assert_eq!(res.zakat_due, dec!(75.0));
    }
    
    #[test]
    fn test_below_nisab() {
         let config = ZakatConfig::default(); // 653kg
         let agri = AgricultureAssets::new(dec!(600.0), dec!(1.0), IrrigationMethod::Rain).unwrap();
         let res = agri.with_hawl(true).calculate_zakat(&config).unwrap();
         
         assert!(!res.is_payable);
    }
}
