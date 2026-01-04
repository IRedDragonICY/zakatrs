//! Zakat Fitrah Calculator
//!
//! Calculates Zakat al-Fitr (Fitrah), the obligatory charity paid before Eid al-Fitr.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Serialize, Deserialize};
use crate::types::{ZakatDetails, ZakatError, InvalidInputDetails, ErrorDetails};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;

#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FitrahCalculator {
    pub person_count: u32,
    pub price_per_unit: Decimal,
    pub unit_amount: Decimal,
    pub label: Option<String>,
    id: uuid::Uuid,
}

impl FitrahCalculator {
    pub fn new(
        person_count: u32,
        price_per_unit: impl IntoZakatDecimal,
        unit_amount: Option<impl IntoZakatDecimal>,
    ) -> Result<Self, ZakatError> {
        let price = price_per_unit.into_zakat_decimal()?;
        let amount = match unit_amount {
            Some(v) => v.into_zakat_decimal()?,
            None => dec!(2.5), // Default 2.5kg
        };

        if person_count == 0 {
            return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "person_count".to_string(),
                value: "0".to_string(),
                reason_key: "error-fitrah-count".to_string(),
                suggestion: Some("Person count must be at least 1.".to_string()),
                ..Default::default()
            })));
        }
        if price < Decimal::ZERO {
            return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "price_per_unit".to_string(),
                value: "negative".to_string(),
                reason_key: "error-negative-value".to_string(),
                suggestion: Some("Zakat inputs must be positive.".to_string()),
                ..Default::default()
            })));
        }

        Ok(Self {
            person_count,
            price_per_unit: price,
            unit_amount: amount,
            label: None,
            id: uuid::Uuid::new_v4(),
        })
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl CalculateZakat for FitrahCalculator {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        let config_cow = config.resolve_config();
        let _config = config_cow.as_ref();

        let total_people_decimal: Decimal = self.person_count.into();
        let total_value = total_people_decimal
            .checked_mul(self.unit_amount)
            .and_then(|v| v.checked_mul(self.price_per_unit))
            .ok_or(ZakatError::CalculationError(Box::new(ErrorDetails {
                reason_key: "error-fitrah-overflow".to_string(),
                code: crate::types::ZakatErrorCode::CalculationOverflow,
                ..Default::default()
            })))?;

        // Build calculation trace
        let trace = vec![
            crate::types::CalculationStep::initial("step-person-count", "Person Count", total_people_decimal),
            crate::types::CalculationStep::initial("step-amount-per-person", "Amount per Person (kg)", self.unit_amount),
            crate::types::CalculationStep::initial("step-price-per-kg", "Price per kg", self.price_per_unit),
            crate::types::CalculationStep::info("info-fitrah-obligatory", "Fitrah is obligatory - no Nisab threshold"),
            crate::types::CalculationStep::result("step-total-fitrah-due", "Total Fitrah Due", total_value),
        ];

        #[allow(deprecated)]
        Ok(ZakatDetails {
            total_assets: total_value,
            liabilities_due_now: Decimal::ZERO,
            liabilities: Vec::new(),
            net_assets: total_value,
            nisab_threshold: Decimal::ZERO, 
            is_payable: true, // Fitrah is obligatory
            zakat_due: total_value,
            wealth_type: crate::types::WealthType::Fitrah,
            status_reason: None,
            label: self.label.clone(),
            payload: crate::types::PaymentPayload::Monetary(total_value),
            calculation_trace: crate::types::CalculationTrace(trace),
            warnings: Vec::new(),
            structured_warnings: Vec::new(),
            recommendation: crate::types::ZakatRecommendation::Obligatory, // Fitrah is always obligatory
        })
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }

    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}

/// Calculates Zakat Fitrah.
///
/// # Arguments
///
/// * `person_count` - Number of people to pay for.
/// * `price_per_unit` - Price of the staple food per unit (kg or liter).
/// * `unit_amount` - Amount per person. Defaults to 2.5 (kg) if None.
///
/// # Returns
///
/// `ZakatDetails` where `zakat_due` is the total monetary value.
pub fn calculate_fitrah(
    person_count: u32,
    price_per_unit: impl IntoZakatDecimal,
    unit_amount: Option<Decimal>,
) -> Result<ZakatDetails, ZakatError> {
    let calculator = FitrahCalculator::new(person_count, price_per_unit, unit_amount)?;
    calculator.calculate_zakat(&ZakatConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fitrah_basic() {
        let price = 10.0; // 10 currency per kg
        let people = 4;
        // Default 2.5kg * 4 people * 10 = 100
        let result = calculate_fitrah(people, price, None).unwrap();
        assert_eq!(result.zakat_due, dec!(100));
        assert!(result.is_payable);
    }

    #[test]
    fn test_fitrah_custom_amount() {
        let price = 2.0;
        let people = 1;
        let amount = dec!(3.5); // 3.5 Using liters or different mazhab
        // 1 * 3.5 * 2 = 7
        let result = calculate_fitrah(people, price, Some(amount)).unwrap();
        assert_eq!(result.zakat_due, dec!(7));
    }
    
    #[test]
    fn test_fitrah_calculator_usage() {
        let calc = FitrahCalculator::new(2, 5.0, None::<Decimal>).unwrap();
        let res = calc.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, dec!(25)); // 2 * 2.5 * 5 = 25
    }
}
