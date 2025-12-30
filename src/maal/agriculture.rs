use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrrigationMethod {
    Rain, // Natural, 10%
    Irrigated, // Artificial/Costly, 5%
    Mixed, // Both, 7.5%
}

impl Default for IrrigationMethod {
    fn default() -> Self {
        Self::Rain
    }
}

#[derive(Default)]
pub struct AgricultureAssets {
    pub harvest_weight_kg: Decimal,
    pub price_per_kg: Decimal,
    pub irrigation: IrrigationMethod,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl AgricultureAssets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new AgricultureAssets instance from Wasaq units.
    /// 1 Wasaq is approximately 130.6 kg.
    pub fn from_wasaq(
        wasaq: impl IntoZakatDecimal,
        price_per_kg: impl IntoZakatDecimal,
        irrigation: IrrigationMethod,
    ) -> Self {
        let mut s = Self::default();
        if let Ok(w) = wasaq.into_zakat_decimal() {
            s.harvest_weight_kg = w * dec!(130.6);
        }
        if let Ok(p) = price_per_kg.into_zakat_decimal() {
            s.price_per_kg = p;
        }
        s.irrigation = irrigation;
        s
    }

    pub fn harvest_weight(mut self, weight: impl IntoZakatDecimal) -> Self {
        if let Ok(w) = weight.into_zakat_decimal() {
            self.harvest_weight_kg = w;
        }
        self
    }

    pub fn price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.price_per_kg = p;
        }
        self
    }

    pub fn irrigation(mut self, irrigation: IrrigationMethod) -> Self {
        self.irrigation = irrigation;
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

impl CalculateZakat for AgricultureAssets {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        if self.harvest_weight_kg < Decimal::ZERO || self.price_per_kg < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Harvest weight and price must be non-negative".to_string(), self.label.clone()));
        }

        let rate = match self.irrigation {
            IrrigationMethod::Rain => dec!(0.10),
            IrrigationMethod::Irrigated => dec!(0.05),
            IrrigationMethod::Mixed => dec!(0.075),
        };
        
        let nisab_threshold_kg = config.get_nisab_agriculture_kg();

        let total_value = self.harvest_weight_kg
            .checked_mul(self.price_per_kg)
            .ok_or(ZakatError::CalculationError("Overflow calculating agriculture total value".to_string(), self.label.clone()))?;
        let nisab_value = nisab_threshold_kg
            .checked_mul(self.price_per_kg)
            .ok_or(ZakatError::CalculationError("Overflow calculating agriculture nisab value".to_string(), self.label.clone()))?; 
        
        let liabilities = self.liabilities_due_now;
        
        // Fiqh Note: Agriculture Nisab is based on the Harvest Quantity (5 Wasqs).
        // However, calculation is done on the monetary value for consistency.
        // We check if (Net Value) >= (Nisab Quantity Value) to determine payability.
        
        let net_value = total_value
            .checked_sub(liabilities)
            .ok_or(ZakatError::CalculationError("Underflow calculating agriculture net value".to_string(), self.label.clone()))?;
        
        let zakat_due = if net_value >= nisab_value {
             net_value
                 .checked_mul(rate)
                 .ok_or(ZakatError::CalculationError("Overflow calculating agriculture zakat due".to_string(), self.label.clone()))?
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
        
        let agri = AgricultureAssets::new()
            .harvest_weight(dec!(1000.0))
            .price(dec!(1.0))
            .irrigation(IrrigationMethod::Rain);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(100.0));
    }

    #[test]
    fn test_agriculture_irrigated() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(dec!(1000.0))
            .price(dec!(1.0))
            .irrigation(IrrigationMethod::Irrigated);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        // Irrigated -> 5%.
        // Due 50.
        assert_eq!(res.zakat_due, dec!(50.0));
    }
    
    #[test]
    fn test_agriculture_mixed() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(dec!(1000.0))
            .price(dec!(1.0))
            .irrigation(IrrigationMethod::Mixed);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        // Mixed -> 7.5%.
        // Due 75.
        assert_eq!(res.zakat_due, dec!(75.0));
    }
    
    #[test]
    fn test_below_nisab() {
         let config = ZakatConfig::default(); // 653kg
         let agri = AgricultureAssets::new()
            .harvest_weight(dec!(600.0))
            .price(dec!(1.0))
            .irrigation(IrrigationMethod::Rain);
            
         let res = agri.hawl(true).calculate_zakat(&config).unwrap();
         
         assert!(!res.is_payable);
    }
    #[test]
    fn test_agriculture_payload() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(dec!(1000.0))
            .price(dec!(1.0))
            .irrigation(IrrigationMethod::Rain);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
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
