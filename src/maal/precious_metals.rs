use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError, WealthType};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub struct PreciousMetal {
    pub weight_grams: Decimal,
    pub metal_type: WealthType, // Gold or Silver
    pub purity: u32, // Karat for Gold (e.g. 24, 21, 18). Ignored for Silver (assumed pure).
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl PreciousMetal {
    pub fn new(weight_grams: impl Into<Decimal>, metal_type: WealthType) -> Result<Self, ZakatError> {
        let weight: Decimal = weight_grams.into();
        if weight < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Weight must be non-negative".to_string()));
        }

        match metal_type {
            WealthType::Gold | WealthType::Silver => {},
            _ => return Err(ZakatError::InvalidInput("Type must be Gold or Silver".to_string())),
        };
        
        Ok(Self {
            weight_grams: weight,
            metal_type,
            purity: 24, // Default to 24K (Pure)
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        })
    }

    pub fn with_debt_due_now(mut self, debt: impl Into<Decimal>) -> Self {
        self.liabilities_due_now = debt.into();
        self
    }

    pub fn with_hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn with_purity(mut self, karat: u32) -> Self {
        self.purity = karat;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl CalculateZakat for PreciousMetal {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        let (price_per_gram, nisab_threshold_grams) = match self.metal_type {
            WealthType::Gold => (config.gold_price_per_gram, config.get_nisab_gold_grams()),
            WealthType::Silver => (config.silver_price_per_gram, config.get_nisab_silver_grams()),
            _ => return Err(ZakatError::InvalidInput("Type must be Gold or Silver".to_string())),
        };

        if price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError("Price for metal not set".to_string()));
        }

        let nisab_value = nisab_threshold_grams * price_per_gram;
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::not_payable(nisab_value, self.metal_type, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        // Normalize weight if it's Gold and not 24K
        let effective_weight = if self.metal_type == WealthType::Gold && self.purity < 24 {
            // formula: weight * (karat / 24)
            self.weight_grams * (Decimal::from(self.purity) / Decimal::from(24))
        } else {
            self.weight_grams
        };

        let total_value = effective_weight * price_per_gram;
        let liabilities = self.liabilities_due_now;

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
        let metal = PreciousMetal::new(dec!(84.0), WealthType::Gold).unwrap();
        let zakat = metal.with_hawl(true).calculate_zakat(&config).unwrap();
        
        // 84g < 85g -> Not Payable
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_above_nisab() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        let metal = PreciousMetal::new(dec!(85.0), WealthType::Gold).unwrap();
        let zakat = metal.with_hawl(true).calculate_zakat(&config).unwrap();
        
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
        
        let metal = PreciousMetal::new(dec!(100.0), WealthType::Gold).unwrap();
        let zakat = metal.with_debt_due_now(dec!(2000.0)).with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_purity_18k() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        
        // 100g of 18K Gold.
        // Effective Weight = 100 * (18/24) = 75g.
        // Nisab = 85g.
        // 75g < 85g -> Not Payable.
        // If it were treated as 24K, it would be payable.
        
        let metal = PreciousMetal::new(dec!(100.0), WealthType::Gold).unwrap()
            .with_purity(18);
            
        let zakat = metal.with_hawl(true).calculate_zakat(&config).unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
        
        // Test 24K explicit
        let metal24 = PreciousMetal::new(dec!(100.0), WealthType::Gold).unwrap()
            .with_purity(24);
        let zakat24 = metal24.with_hawl(true).calculate_zakat(&config).unwrap();
        assert!(zakat24.is_payable);
    }
}
