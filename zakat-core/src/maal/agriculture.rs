//! # Fiqh Compliance: Agriculture
//!
//! ## Rates
//! - **10% (Ushr)**: Rain-fed/Natural irrigation. (Source: Sahih Bukhari 1483).
//! - **5% (Half-Ushr)**: Irrigated/Labor-intensive. (Source: Sahih Muslim 981).
//! - **7.5%**: Mixed irrigation methods (derived via Ijtihad).
//!
//! ## Nisab
//! - **Threshold**: 5 Awsuq. (Source: Sahih Muslim 979).
//! - **Conversion**: Configurable, defaults to **653 kg** based on the research of Dr. Yusuf Al-Qaradawi (*Fiqh al-Zakah*).

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::validation::Validator;

use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum IrrigationMethod {
    #[default]
    Rain, // Natural, 10%
    Irrigated, // Artificial/Costly, 5%
    Mixed, // Both, 7.5%
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AgricultureAssets {
    pub harvest_weight_kg: Decimal,
    pub price_per_kg: Decimal,
    pub irrigation: IrrigationMethod,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
    pub id: uuid::Uuid,
}

impl AgricultureAssets {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            ..Default::default()
        }
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
        s.id = uuid::Uuid::new_v4();
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
    
    pub fn validate(&self) -> Result<(), ZakatError> {
        Ok(())
    }
}

impl CalculateZakat for AgricultureAssets {
    fn validate_input(&self) -> Result<(), ZakatError> {
        self.validate()
    }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        Validator::ensure_non_negative(&[
            ("harvest_weight", self.harvest_weight_kg),
            ("price", self.price_per_kg)
        ], self.label.clone())?;

        let rate = match self.irrigation {
            IrrigationMethod::Rain => dec!(0.10),
            IrrigationMethod::Irrigated => dec!(0.05),
            IrrigationMethod::Mixed => dec!(0.075),
        };
        
        let nisab_threshold_kg = config.get_nisab_agriculture_kg();

        let total_value = ZakatDecimal::new(self.harvest_weight_kg)
            .safe_mul(self.price_per_kg)?
            .with_source(self.label.clone());

        let nisab_value = ZakatDecimal::new(nisab_threshold_kg)
            .safe_mul(self.price_per_kg)?
            .with_source(self.label.clone()); 
        
        let liabilities = self.liabilities_due_now;
        
        // Fiqh Note: Agriculture Nisab is based on the Harvest Quantity (5 Wasqs).
        // However, calculation is done on the monetary value for consistency.
        // We check if (Net Value) >= (Nisab Quantity Value) to determine payability.
        
        // Use *total_value to get Decimal for creating ZakatDecimal again, or implement methods on ZakatDecimal to take ZakatDecimal
        // safe_sub takes impl Into<Decimal>, so passing ZakatDecimal works (it implements Into<Decimal>).
        let net_value = total_value
            .safe_sub(liabilities)?
            .with_source(self.label.clone());
        
        let zakat_due = if *net_value >= *nisab_value {
             net_value
                 .safe_mul(rate)?
                 .with_source(self.label.clone())
        } else {
             ZakatDecimal::default()
        };

        let is_payable = *zakat_due > Decimal::ZERO;

        // Build calculation trace
        let irrigation_desc = match self.irrigation {
            IrrigationMethod::Rain => "Rain-fed (10%)",
            IrrigationMethod::Irrigated => "Irrigated (5%)",
            IrrigationMethod::Mixed => "Mixed irrigation (7.5%)",
        };
        
        let mut trace = vec![
            crate::types::CalculationStep::initial("step-harvest-weight", "Harvest Weight (kg)", self.harvest_weight_kg),
            crate::types::CalculationStep::initial("step-price-per-kg", "Price per kg", self.price_per_kg),
            crate::types::CalculationStep::result("step-total-harvest-value", "Total Harvest Value", *total_value),
            crate::types::CalculationStep::subtract("step-debts-due-now", "Liabilities Due Now", liabilities),
            crate::types::CalculationStep::result("step-net-harvest-value", "Net Harvest Value", *net_value),
            crate::types::CalculationStep::compare("step-nisab-check-value", "Nisab Threshold (653kg value)", *nisab_value),
        ];
        if is_payable {
            trace.push(crate::types::CalculationStep::info("info-irrigation-method", format!("Irrigation Method: {}", irrigation_desc))
                 .with_args(std::collections::HashMap::from([("method".to_string(), irrigation_desc.to_string())])));
            trace.push(crate::types::CalculationStep::rate("step-rate-applied", "Applied Rate", rate));
            trace.push(crate::types::CalculationStep::result("step-zakat-due", "Zakat Due", *zakat_due));
        } else {
            trace.push(crate::types::CalculationStep::info("status-exempt", "Net Value below Nisab - No Zakat Due"));
        }

        #[allow(deprecated)]
        Ok(ZakatDetails {
            total_assets: *total_value,
            liabilities_due_now: liabilities,
            liabilities: Vec::new(),
            net_assets: *net_value,
            nisab_threshold: *nisab_value,
            is_payable,
            zakat_due: *zakat_due,
            wealth_type: crate::types::WealthType::Agriculture,
            status_reason: None,
            label: self.label.clone(),
            payload: crate::types::PaymentPayload::Agriculture {
                harvest_weight: self.harvest_weight_kg,
                irrigation_method: irrigation_desc.to_string(),
                crop_value: *zakat_due,
            },
            calculation_trace: crate::types::CalculationTrace(trace),
            warnings: Vec::new(),
            structured_warnings: Vec::new(),
            recommendation: if is_payable { 
                crate::types::ZakatRecommendation::Obligatory 
            } else { 
                crate::types::ZakatRecommendation::None 
            },
        })
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
    use crate::config::ZakatConfig;

    #[test]
    fn test_agriculture_rain() {
        let config = ZakatConfig::default(); // default 653kg
        // 1000 > 653. 
        // Rain -> 10%.
        // Price 1.0 -> Value 1000.
        // Due 100.
        
        let agri = AgricultureAssets::new()
            .harvest_weight(1000.0)
            .price(1.0)
            .irrigation(IrrigationMethod::Rain);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(100));
    }

    #[test]
    fn test_agriculture_irrigated() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(1000.0)
            .price(1.0)
            .irrigation(IrrigationMethod::Irrigated);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        // Irrigated -> 5%.
        // Due 50.
        assert_eq!(res.zakat_due, Decimal::from(50));
    }
    
    #[test]
    fn test_agriculture_mixed() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(1000.0)
            .price(1.0)
            .irrigation(IrrigationMethod::Mixed);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        // Mixed -> 7.5%.
        // Due 75.
        assert_eq!(res.zakat_due, Decimal::from(75));
    }
    
    #[test]
    fn test_below_nisab() {
         let config = ZakatConfig::default(); // 653kg
         let agri = AgricultureAssets::new()
            .harvest_weight(600.0)
            .price(1.0)
            .irrigation(IrrigationMethod::Rain);
            
         let res = agri.hawl(true).calculate_zakat(&config).unwrap();
         
         assert!(!res.is_payable);
    }
    #[test]
    fn test_agriculture_payload() {
        let config = ZakatConfig::default();
        let agri = AgricultureAssets::new()
            .harvest_weight(1000.0)
            .price(1.0)
            .irrigation(IrrigationMethod::Rain);
            
        let res = agri.hawl(true).calculate_zakat(&config).unwrap();
        
        match res.payload {
            crate::types::PaymentPayload::Agriculture { harvest_weight, irrigation_method, crop_value } => {
                assert_eq!(harvest_weight, Decimal::from(1000));
                assert_eq!(irrigation_method, "Rain-fed (10%)");
                assert_eq!(crop_value, Decimal::from(100));
            },
            _ => panic!("Expected Agriculture payload"),
        }
    }
}
