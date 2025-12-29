use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError, WealthType};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub struct PreciousMetal {
    pub weight_grams: Decimal,
    pub metal_type: WealthType, // Gold or Silver
    pub price_per_gram: Decimal,
    pub nisab_threshold_grams: Decimal,
    pub deductible_liabilities: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl PreciousMetal {
    pub fn new(weight_grams: impl Into<Decimal>, metal_type: WealthType, config: &ZakatConfig) -> Result<Self, ZakatError> {
        let weight: Decimal = weight_grams.into();
        if weight < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Weight must be non-negative".to_string()));
        }

        let (price_per_gram, nisab_threshold_grams) = match metal_type {
            WealthType::Gold => (config.gold_price_per_gram, config.get_nisab_gold_grams()),
            WealthType::Silver => (config.silver_price_per_gram, config.get_nisab_silver_grams()),
            _ => return Err(ZakatError::InvalidInput("Type must be Gold or Silver".to_string())),
        };
        
        if price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError("Price for metal not set".to_string()));
        }

        Ok(Self {
            weight_grams: weight,
            metal_type,
            price_per_gram,
            nisab_threshold_grams,
            deductible_liabilities: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        })
    }

    pub fn with_debt(mut self, debt: impl Into<Decimal>) -> Self {
        self.deductible_liabilities = debt.into();
        self
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

impl CalculateZakat for PreciousMetal {
    fn calculate_zakat(&self) -> Result<ZakatDetails, ZakatError> {
        let nisab_value = self.nisab_threshold_grams * self.price_per_gram;
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::not_payable(nisab_value, self.metal_type, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        let total_value = self.weight_grams * self.price_per_gram;
        let liabilities = self.deductible_liabilities;

        // Note: For Gold/Silver, usually debts are deducted from the wealth itself, 
        // or rather, we check if net wealth >= nisab.
        // The implementation plan says: "Deductible Rule: Allow passing 'Current Debts' to be deducted from assets before checking Nisab"
        
        let rate = dec!(0.025); // 2.5%

        Ok(ZakatDetails::new(total_value, liabilities, nisab_value, rate, self.metal_type)
            .with_label(self.label.clone().unwrap_or_default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gold_below_nisab() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        let metal = PreciousMetal::new(dec!(84.0), WealthType::Gold, &config).unwrap();
        let zakat = metal.with_hawl(true).calculate_zakat().unwrap();
        
        // 84g < 85g -> Not Payable
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_above_nisab() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        let metal = PreciousMetal::new(dec!(85.0), WealthType::Gold, &config).unwrap();
        let zakat = metal.with_hawl(true).calculate_zakat().unwrap();
        
        // 85g >= 85g -> Payable
        // Value = 8500
        // Due = 8500 * 0.025 = 212.5
        assert!(zakat.is_payable);
        assert_eq!(zakat.zakat_due, dec!(212.5));
    }

    #[test]
    fn test_gold_with_debt() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        // 100g Gold ($10,000). Debt $2,000. Net $8,000.
        // Nisab 85g = $8,500.
        // Net ($8,000) < Nisab ($8,500) -> Not Payable.
        
        let metal = PreciousMetal::new(dec!(100.0), WealthType::Gold, &config).unwrap();
        let zakat = metal.with_debt(dec!(2000.0)).with_hawl(true).calculate_zakat().unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }
}
