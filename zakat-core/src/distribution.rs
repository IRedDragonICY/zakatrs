//! # Zakat Distribution (Haddul Kifayah)
//!
//! Determines eligibility to RECEIVE Zakat based on local poverty lines and sufficiency limits.

use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum EligibilityStatus {
    Fakir,
    Miskin,
    NotEligible,
}

/// Basic monthly needs cost for a reference individual/household in a specific location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicNeeds {
    pub food: Decimal,
    pub shelter: Decimal,
    pub clothing: Decimal,
    pub education: Decimal,
    pub health: Decimal,
    pub transport: Decimal,
    pub utilities: Decimal,
}

/// Profile of a household applying for Zakat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdProfile {
    pub monthly_income: Decimal,
    pub dependents_count: u32,
    pub location_factor: Decimal, 
}

impl Default for BasicNeeds {
    fn default() -> Self {
        use rust_decimal_macros::dec;
        Self {
            food: dec!(300),
            shelter: dec!(500),
            clothing: dec!(50),
            education: dec!(100),
            health: dec!(50),
            transport: dec!(100),
            utilities: dec!(100),
        }
    }
}

impl BasicNeeds {
    pub fn total(&self) -> Decimal {
        self.food + self.shelter + self.clothing + self.education + self.health + self.transport + self.utilities
    }
}

impl HouseholdProfile {
    pub fn new(income: impl Into<Decimal>, dependents: u32) -> Self {
        Self {
            monthly_income: income.into(),
            dependents_count: dependents,
            location_factor: Decimal::ONE,
        }
    }
    
    pub fn with_location_factor(mut self, factor: impl Into<Decimal>) -> Self {
        self.location_factor = factor.into();
        self
    }
}

pub struct DistributionCalculator;

impl DistributionCalculator {
    pub fn calculate_haddul_kifayah(needs: &BasicNeeds, profile: &HouseholdProfile) -> Decimal {
        let base_total = needs.total();
        let adjusted_base = base_total * profile.location_factor;
        
        let dependent_cost = adjusted_base * rust_decimal_macros::dec!(0.5);
        let total_dependent_cost = dependent_cost * Decimal::from(profile.dependents_count);
        
        adjusted_base + total_dependent_cost
    }

    pub fn check_eligibility(needs: &BasicNeeds, profile: &HouseholdProfile) -> EligibilityStatus {
        let kifiyah_limit = Self::calculate_haddul_kifayah(needs, profile);
        
        if kifiyah_limit.is_zero() {
            return EligibilityStatus::NotEligible; 
        }

        let sufficiency_ratio = profile.monthly_income / kifiyah_limit;
        
        if sufficiency_ratio < rust_decimal_macros::dec!(0.5) {
            EligibilityStatus::Fakir
        } else if sufficiency_ratio < Decimal::ONE {
            EligibilityStatus::Miskin
        } else {
            EligibilityStatus::NotEligible
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_eligibility_calculator() {
        let needs = BasicNeeds {
            food: dec!(200),
            shelter: dec!(300), 
            ..Default::default()
        }; 
        let profile = HouseholdProfile::new(dec!(500), 0);
        let status = DistributionCalculator::check_eligibility(&needs, &profile);
        assert_eq!(status, EligibilityStatus::Miskin);
    }
}
