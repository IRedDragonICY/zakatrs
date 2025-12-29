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
            return Err(ZakatError::InvalidInput("Harvest weight and price must be non-negative".to_string(), None));
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

        let total_value = self.harvest_weight_kg
            .checked_mul(self.price_per_kg)
            .ok_or(ZakatError::CalculationError("Overflow calculating agriculture total value".to_string(), None))?;
        let nisab_value = nisab_threshold_kg
            .checked_mul(self.price_per_kg)
            .ok_or(ZakatError::CalculationError("Overflow calculating agriculture nisab value".to_string(), None))?; 
        
        let liabilities = self.liabilities_due_now;
        
        // Fiqh Note: Agriculture Nisab is based on the Harvest Quantity (5 Wasqs).
        // However, calculation is done on the monetary value for consistency.
        // We check if (Net Value) >= (Nisab Quantity Value) to determine payability.
        
        let net_value = total_value
            .checked_sub(liabilities)
            .ok_or(ZakatError::CalculationError("Underflow calculating agriculture net value".to_string(), None))?;
        
        let zakat_due = if net_value >= nisab_value {
             net_value
                 .checked_mul(rate)
                 .ok_or(ZakatError::CalculationError("Overflow calculating agriculture zakat due".to_string(), None))?
        } else {
             Decimal::ZERO
        };

        let is_payable = zakat_due > Decimal::ZERO;

        // Build calculation trace
        let irrigation_desc = match self.irrigation {
            IrrigationMethod::Rain => "Rain-fed (10%)",
            IrrigationMethod::Irrigated => "Irrigated (5%)",
            IrrigationMethod::Mixed => "Mixed irrigation (7.5%)",
        };
        
        let mut trace = vec![
            crate::types::CalculationStep::initial("Harvest Weight (kg)", self.harvest_weight_kg),
            crate::types::CalculationStep::initial("Price per kg", self.price_per_kg),
            crate::types::CalculationStep::result("Total Harvest Value", total_value),
            crate::types::CalculationStep::subtract("Liabilities Due Now", liabilities),
            crate::types::CalculationStep::result("Net Harvest Value", net_value),
            crate::types::CalculationStep::compare("Nisab Threshold (653kg value)", nisab_value),
        ];
        if is_payable {
            trace.push(crate::types::CalculationStep::info(format!("Irrigation Method: {}", irrigation_desc)));
            trace.push(crate::types::CalculationStep::rate("Applied Rate", rate));
            trace.push(crate::types::CalculationStep::result("Zakat Due", zakat_due));
        } else {
            trace.push(crate::types::CalculationStep::info("Net Value below Nisab - No Zakat Due"));
        }

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
            payload: crate::types::PaymentPayload::Agriculture {
                harvest_weight: self.harvest_weight_kg,
                irrigation_method: irrigation_desc.to_string(),
                crop_value: zakat_due,
            },
            calculation_trace: trace,
        })
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
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
    #[test]
    fn test_agriculture_payload() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Rain).unwrap();
        let res = agri.with_hawl(true).calculate_zakat(&config).unwrap();
        
        match res.payload {
            crate::types::PaymentPayload::Agriculture { harvest_weight, irrigation_method, crop_value } => {
                assert_eq!(harvest_weight, dec!(1000.0));
                assert_eq!(irrigation_method, "Rain-fed (10%)");
                assert_eq!(crop_value, dec!(100.0));
            },
            _ => panic!("Expected Agriculture payload"),
        }
    }
}
