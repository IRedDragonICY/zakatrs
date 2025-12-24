use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

pub enum IrrigationMethod {
    Rain, // Natural, 10%
    Irrigated, // Artificial/Costly, 5%
    Mixed, // Both, 7.5%
}

pub struct AgricultureAssets {
    pub harvest_weight_kg: Decimal,
    pub price_per_kg: Decimal,
    pub irrigation: IrrigationMethod,
    pub nisab_threshold_kg: Decimal,
}

impl AgricultureAssets {
    pub fn new(
        harvest_weight_kg: Decimal,
        price_per_kg: Decimal,
        irrigation: IrrigationMethod,
        config: &ZakatConfig, 
    ) -> Result<Self, ZakatError> {
        // Nisab: 5 Wasq. 1 Wasq ~ 60 Sa'. 1 Sa' ~ 2.176 kg (varies but commonly ~653kg total).
        // Requirement says "~653 kg". Use config.
        let nisab = config.get_nisab_agriculture_kg();
        
        if harvest_weight_kg < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Harvest weight cannot be negative".to_string()));
        }

        Ok(Self {
            harvest_weight_kg,
            price_per_kg,
            irrigation,
            nisab_threshold_kg: nisab,
        })
    }
}

impl CalculateZakat for AgricultureAssets {
    fn calculate_zakat(&self, extra_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        let rate = match self.irrigation {
            IrrigationMethod::Rain => dec!(0.10),
            IrrigationMethod::Irrigated => dec!(0.05),
            IrrigationMethod::Mixed => dec!(0.075),
        };
        
        let total_value = self.harvest_weight_kg * self.price_per_kg;
        let nisab_value = self.nisab_threshold_kg * self.price_per_kg; // Nisab is strictly on weight, but we convert for ZakatDetails
        
        // Agriculture usually doesn't deduct debts in some madhabs, but requirements said allow flexible.
        let liabilities = extra_debts.unwrap_or(Decimal::ZERO);
        
        // Check Nisab by Weight
        let is_nisab_reached = self.harvest_weight_kg >= self.nisab_threshold_kg;
        
        // ZakatDetails logic overrides 'is_payable' based on net_assets usually, but here Nisab is on Quantity.
        // We need to be careful. If Quantity > Nisab, then we pay rate on Total.
        // If we strictly follow "Net Assets" logic, we would deduct debt from value first.
        // But for Agriculture, usually checks yield quantity first.
        // Let's ensure ZakatDetails reflects the state correctly.
        
        let net_value = total_value - liabilities; // Just for reporting
        
        let zakat_due = if is_nisab_reached {
             // Rate applied to the TOTAL Harvest, not net after debt usually, 
             // but if we support debt deduction, we might apply to net.
             // Given the complexity of Fiqh here, the safest implementation of "Deductible Rule" requested:
             // "Allow passing ... deducted from assets before checking Nisab"
             // Implementation: (Gross - Debt) >= Nisab ? 
             // Wait, mixing weight and value. Debt is Value. Harvest is Weight.
             // We convert Harvest to Value to check Nisab? 
             // Or we deduct equivalent weight?
             // Simplest interpretation: Deduct debt value from Total Value, then check if remaining Value >= Nisab Value.
             
             if net_value >= nisab_value {
                 net_value * rate
             } else {
                 Decimal::ZERO
             }
        } else {
            Decimal::ZERO
        };

        // If simple weight check was enough:
        // let zakat_due = if self.harvest_weight_kg >= self.nisab_threshold_kg { total_value * rate } else { Decimal::ZERO };
        // But the user insisted on Debt Deduction.
        // Let's stick to Value based calculation for consistency with debts.
        
        let is_payable = zakat_due > Decimal::ZERO;

        Ok(ZakatDetails {
            total_assets: total_value,
            deductible_liabilities: liabilities,
            net_assets: net_value,
            nisab_threshold: nisab_value,
            is_payable,
            zakat_due,
            wealth_type: crate::types::WealthType::Agriculture,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agriculture_rain() {
        // 1000kg. Price 1. Value 1000.
        // Nisab 653. Reached.
        // Rate 10%. Due 100.
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Rain, &ZakatConfig::default()).unwrap();
        let res = agri.calculate_zakat(None).unwrap();
        
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(100.0));
    }

    #[test]
    fn test_agriculture_irrigated() {
        // 1000kg. Price 1. 
        // Rate 5%. Due 50.
        let agri = AgricultureAssets::new(dec!(1000.0), dec!(1.0), IrrigationMethod::Irrigated, &ZakatConfig::default()).unwrap();
        let res = agri.calculate_zakat(None).unwrap();
        
        assert_eq!(res.zakat_due, dec!(50.0));
    }
}
