use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError, WealthType};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

use crate::inputs::IntoZakatDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JewelryUsage {
    Investment,    // Always Zakatable
    PersonalUse,   // Exempt in Shafi/Maliki/Hanbali usually
}

pub struct PreciousMetals {
    pub weight_grams: Decimal,
    pub metal_type: Option<WealthType>, // Gold or Silver
    pub purity: u32, // Karat for Gold (e.g. 24, 21, 18). Ignored for Silver (assumed pure).
    pub usage: JewelryUsage,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl Default for PreciousMetals {
    fn default() -> Self {
        Self {
            weight_grams: Decimal::ZERO,
            metal_type: None,
            purity: 24,
            usage: JewelryUsage::Investment,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        }
    }
}

impl PreciousMetals {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn weight(mut self, weight: impl IntoZakatDecimal) -> Self {
        if let Ok(w) = weight.into_zakat_decimal() {
            self.weight_grams = w;
        }
        self
    }

    pub fn metal_type(mut self, metal_type: WealthType) -> Self {
        self.metal_type = Some(metal_type);
        self
    }

    pub fn purity(mut self, purity: u32) -> Self {
        self.purity = purity;
        self
    }

    pub fn usage(mut self, usage: JewelryUsage) -> Self {
        self.usage = usage;
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

impl CalculateZakat for PreciousMetals {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        let metal_type = self.metal_type.ok_or_else(|| 
            ZakatError::InvalidInput("Metal type must be specified (Gold or Silver)".to_string(), self.label.clone())
        )?;

        if self.weight_grams < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Weight must be non-negative".to_string(), self.label.clone()));
        }

        match metal_type {
            WealthType::Gold | WealthType::Silver => {},
            _ => return Err(ZakatError::InvalidInput("Type must be Gold or Silver".to_string(), self.label.clone())),
        };

        // Check for personal usage exemption first
        if self.usage == JewelryUsage::PersonalUse && config.madhab.strategy().get_rules().jewelry_exempt {
             return Ok(ZakatDetails::below_threshold(
                 Decimal::ZERO, 
                 metal_type, 
                 "Personal jewelry is exempt in this Madhab"
             ).with_label(self.label.clone().unwrap_or_default()));
        }

        let (price_per_gram, nisab_threshold_grams) = match metal_type {
            WealthType::Gold => (config.gold_price_per_gram, config.get_nisab_gold_grams()),
            WealthType::Silver => (config.silver_price_per_gram, config.get_nisab_silver_grams()),
            _ => return Err(ZakatError::InvalidInput("Type must be Gold or Silver".to_string(), self.label.clone())),
        };

        if price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError("Price for metal not set".to_string(), self.label.clone()));
        }

        let nisab_value = nisab_threshold_grams
            .checked_mul(price_per_gram)
            .ok_or(ZakatError::CalculationError("Overflow calculating metal nisab value".to_string(), self.label.clone()))?;
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(nisab_value, metal_type, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        // Normalize weight if it's Gold and not 24K
        let effective_weight = if metal_type == WealthType::Gold && self.purity < 24 {
            // formula: weight * (karat / 24)
            let purity_ratio = Decimal::from(self.purity)
                .checked_div(Decimal::from(24))
                .ok_or(ZakatError::CalculationError("Error calculating purity ratio".to_string(), self.label.clone()))?;
            self.weight_grams
                .checked_mul(purity_ratio)
                .ok_or(ZakatError::CalculationError("Overflow calculating effective gold weight".to_string(), self.label.clone()))?
        } else {
            self.weight_grams
        };

        let total_value = effective_weight
            .checked_mul(price_per_gram)
            .ok_or(ZakatError::CalculationError("Overflow calculating metal total value".to_string(), self.label.clone()))?;
        let liabilities = self.liabilities_due_now;

        let rate = dec!(0.025); // 2.5%

        // Build calculation trace
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial("Weight (grams)", self.weight_grams));
        trace.push(crate::types::CalculationStep::initial("Price per gram", price_per_gram));
        
        if metal_type == crate::types::WealthType::Gold && self.purity < 24 {
             trace.push(crate::types::CalculationStep::info(format!("Purity Adjustment ({}K / 24K)", self.purity)));
             trace.push(crate::types::CalculationStep::result("Effective 24K Weight", effective_weight));
        }
        
        trace.push(crate::types::CalculationStep::result("Total Value", total_value));
        trace.push(crate::types::CalculationStep::subtract("Debts Due Now", liabilities));
        
        let net_val = total_value - liabilities;
        trace.push(crate::types::CalculationStep::result("Net Value", net_val));
        trace.push(crate::types::CalculationStep::compare("Nisab Threshold", nisab_value));

        if net_val >= nisab_value && net_val > Decimal::ZERO {
            trace.push(crate::types::CalculationStep::rate("Applied Rate (2.5%)", rate));
        } else {
             trace.push(crate::types::CalculationStep::info("Net Value below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails::with_trace(total_value, liabilities, nisab_value, rate, metal_type, trace)
            .with_label(self.label.clone().unwrap_or_default()))
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::madhab::Madhab;

    #[test]
    fn test_gold_below_nisab() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        let metal = PreciousMetals::new()
            .weight(dec!(84.0))
            .metal_type(WealthType::Gold)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        // 84g < 85g -> Not Payable
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
    }

    #[test]
    fn test_gold_above_nisab() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        let metal = PreciousMetals::new()
            .weight(dec!(85.0))
            .metal_type(WealthType::Gold)
            .hawl(true);
        let zakat = metal.calculate_zakat(&config).unwrap();
        
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
        
        let metal = PreciousMetals::new()
            .weight(dec!(100.0))
            .metal_type(WealthType::Gold)
            .debt(dec!(2000.0))
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
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
        
        let metal = PreciousMetals::new()
            .weight(dec!(100.0))
            .metal_type(WealthType::Gold)
            .purity(18)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        
        assert!(!zakat.is_payable);
        assert_eq!(zakat.zakat_due, Decimal::ZERO);
        
        // Test 24K explicit
        let metal24 = PreciousMetals::new()
            .weight(dec!(100.0))
            .metal_type(WealthType::Gold)
            .purity(24)
            .hawl(true);
        let zakat24 = metal24.calculate_zakat(&config).unwrap();
        assert!(zakat24.is_payable);
    }
    #[test]
    fn test_personal_jewelry_hanafi_payable() {
        // Hanafi uses LowerOfTwo. Personal jewelry is Zakatable.
        let config = ZakatConfig { 
            gold_price_per_gram: dec!(100.0), 
            madhab: Madhab::Hanafi,
            ..Default::default() 
        };
        
        // 100g > 85g Nisab
        let metal = PreciousMetals::new()
            .weight(dec!(100.0))
            .metal_type(WealthType::Gold)
            .usage(JewelryUsage::PersonalUse)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        assert!(zakat.is_payable);
    }

    #[test]
    fn test_personal_jewelry_shafi_exempt() {
        // Shafi uses Gold Standard. Personal jewelry is Exempt.
        let config = ZakatConfig { 
            gold_price_per_gram: dec!(100.0), 
            madhab: Madhab::Shafi,
            ..Default::default() 
        };
        
        let metal = PreciousMetals::new()
            .weight(dec!(100.0))
            .metal_type(WealthType::Gold)
            .usage(JewelryUsage::PersonalUse)
            .hawl(true);
            
        let zakat = metal.calculate_zakat(&config).unwrap();
        assert!(!zakat.is_payable);
        assert_eq!(zakat.status_reason, Some("Personal jewelry is exempt in this Madhab".to_string()));
    }
}
