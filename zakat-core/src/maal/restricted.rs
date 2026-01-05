//! # Fiqh Compliance: Restricted Funds (Daman Ijtima'i / Pension)
//!
//! Handles Zakat on assets that are not fully accessible ("Milk Tam"), such as:
//! - Pension Plans (401k, Superannuation, EPF/KWSP, BPJS)
//! - Social Security
//! - Escrow Accounts
//!
//! ## Fiqh Rulings
//! 1. **Fully Accessible**: Zakatable immediately on total value.
//! 2. **Penalty Withdrawal**: Zakatable on (Total - Penalty).
//! 3. **Locked Until Retirement**:
//!    - *Opinion A (Majority/Qaradawi)*: No Zakat until received. Then pay for one year (or all years).
//!    - *Opinion B (Conservative)*: Pay Zakat annually on the *vested* amount (amount you legally own even if fired).

use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use crate::types::{ZakatDetails, ZakatError, WealthType};
use crate::math::ZakatDecimal;
use crate::traits::{CalculateZakat, ZakatConfigArgument};

use crate::maal::calculator::{calculate_monetary_asset, MonetaryCalcParams};
use crate::inputs::{ToFfiString, FromFfiString};

/// Accessibility level of the restricted fund.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum AccessibilityLevel {
    /// Can withdraw anytime without penalty (e.g. voluntary savings).
    /// **Ruling**: Treat as Cash.
    FullyAccessible,

    /// Can withdraw but with a fine/penalty (e.g. 401k early withdrawal).
    /// **Ruling**: Deduct penalty, then treat as Cash.
    PenaltyWithdrawal,

    /// Cannot withdraw until a specific condition (age/retirement) is met.
    /// **Ruling**: Subject to difference of opinion (Qaradawi vs Conservative).
    LockedUntilRetirement,
}

impl ToFfiString for AccessibilityLevel {
    fn to_ffi_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl FromFfiString for AccessibilityLevel {
    type Err = ZakatError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
         match s {
             "FullyAccessible" => Ok(AccessibilityLevel::FullyAccessible),
             "PenaltyWithdrawal" => Ok(AccessibilityLevel::PenaltyWithdrawal),
             "LockedUntilRetirement" => Ok(AccessibilityLevel::LockedUntilRetirement),
             _ => Err(ZakatError::InvalidInput(Box::new(crate::types::InvalidInputDetails {
                 field: "accessibility".to_string(),
                 value: s.to_string(),
                 reason_key: "error-invalid-accessibility".to_string(),
                 ..Default::default()
             })))
         }
    }
}

zakat_ffi_export! {
    /// Represents a Restricted Fund Asset (Pension, 401k, etc.).
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RestrictedFund {
        pub total_value: Decimal,
        pub vested_amount: Decimal,
        pub accessibility: AccessibilityLevel,
        pub withdrawal_penalty: Decimal, // Only relevant for PenaltyWithdrawal
    }
}

impl Default for RestrictedFund {
    fn default() -> Self {
        let (liabilities_due_now, named_liabilities, hawl_satisfied, label, id, _input_errors, acquisition_date) = Self::default_common();
        Self {
            total_value: Decimal::ZERO,
            vested_amount: Decimal::ZERO,
            accessibility: AccessibilityLevel::LockedUntilRetirement,
            withdrawal_penalty: Decimal::ZERO,
            liabilities_due_now,
            named_liabilities,
            hawl_satisfied,
            label,
            id,
            acquisition_date,
            _input_errors,
        }
    }
}

impl RestrictedFund {
    pub fn new_pension(total: impl crate::inputs::IntoZakatDecimal, accessibility: AccessibilityLevel) -> Self {
         Self::new()
            .total_value(total)
            .accessibility(accessibility)
            .hawl(true)
    }

    pub fn total_value(mut self, value: impl crate::inputs::IntoZakatDecimal) -> Self {
        match value.into_zakat_decimal() {
             Ok(v) => {
                 self.total_value = v;
                 // Default vested to total if not set explicitly yet? No, keep separate.
             },
             Err(e) => self._input_errors.push(e),
        }
        self
    }

    pub fn vested_amount(mut self, value: impl crate::inputs::IntoZakatDecimal) -> Self {
        match value.into_zakat_decimal() {
             Ok(v) => self.vested_amount = v,
             Err(e) => self._input_errors.push(e),
        }
        self
    }

    pub fn accessibility(mut self, level: AccessibilityLevel) -> Self {
        self.accessibility = level;
        self
    }

    pub fn penalty(mut self, penalty: impl crate::inputs::IntoZakatDecimal) -> Self {
         match penalty.into_zakat_decimal() {
             Ok(v) => self.withdrawal_penalty = v,
             Err(e) => self._input_errors.push(e),
        }
        self
    }
}

impl CalculateZakat for RestrictedFund {
    fn validate_input(&self) -> Result<(), ZakatError> {
        self.validate()
    }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.validate()?;
        
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        // 1. Determine Zakatable Amount based on Accessibility
        let (zakatable_amount, note) = match self.accessibility {
            AccessibilityLevel::FullyAccessible => {
                (self.total_value, "Fully accessible fund (treated as cash)")
            },
            AccessibilityLevel::PenaltyWithdrawal => {
                let net = self.total_value - self.withdrawal_penalty;
                 if net < Decimal::ZERO {
                    (Decimal::ZERO, "Withdrawal penalty exceeds value")
                } else {
                    (net, "Net value after withdrawal penalty")
                }
            },
            AccessibilityLevel::LockedUntilRetirement => {
                if config.strategy.get_rules().pension_zakat_on_vested {
                     // Conservative/Singapore MUIS: Pay on vested amount annually
                     (self.vested_amount, "Conservative Opinion: Zakat on Vested Amount")
                } else {
                     // Qaradawi/Mainstream: No Zakat until received
                     (Decimal::ZERO, "Standard Opinion: No Zakat until possession (pay upon receipt)")
                }
            }
        };

        let nisab = config.get_monetary_nisab_threshold();
        let rate = config.strategy.get_rules().savings_rate; // Treat as savings

        let total_assets = ZakatDecimal::new(zakatable_amount).with_source(self.label.clone());
        let trace_steps = vec![
            crate::types::CalculationStep::initial("step-total-value", "Total Fund Value", self.total_value),
            crate::types::CalculationStep::result("step-zakatable-base", note, zakatable_amount),
        ];

        let params = MonetaryCalcParams {
            total_assets: *total_assets,
            liabilities: self.total_liabilities(),
            nisab_threshold: nisab,
            rate,
            wealth_type: WealthType::Investment, // Broadly cash/savings
            label: self.label.clone(),
            hawl_satisfied: self.hawl_satisfied,
            asset_id: Some(self.id),
            trace_steps,
            warnings: Vec::new(),
            observer: Some(config.observer.clone()),
        };

        let mut result = calculate_monetary_asset(params)?;
        
        // Add note about the ruling used
        if self.accessibility == AccessibilityLevel::LockedUntilRetirement {
            if result.net_assets.is_zero() {
                 result.notes.push("Funds are locked. Zakat is not due annually according to the majority opinion. Calculate cummulatively when you receive the payout.".to_string());
            } else {
                 result.notes.push("Calculated based on Vested Amount (Conservative Opinion).".to_string());
            }
        }

        Ok(result)
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
    use rust_decimal_macros::dec;

    #[test]
    fn test_locked_fund_majority_opinion() {
        let config = ZakatConfig { gold_price_per_gram: Decimal::from(100), ..Default::default() };


        let fund = RestrictedFund::new_pension(100000.0, AccessibilityLevel::LockedUntilRetirement)
            .vested_amount(50000.0);

        let result = fund.calculate_zakat(&config).unwrap();
        assert!(!result.is_payable);
        assert_eq!(result.zakat_due, Decimal::ZERO);
        assert!(result.notes[0].contains("Funds are locked"));
    }

    #[test]
    fn test_locked_fund_conservative_opinion() {
        let config = ZakatConfig { gold_price_per_gram: Decimal::from(100), ..Default::default() }
            .with_madhab(crate::madhab::Madhab::Shafi);

        let fund = RestrictedFund::new_pension(100000.0, AccessibilityLevel::LockedUntilRetirement)
            .vested_amount(50000.0);

        let result = fund.calculate_zakat(&config).unwrap();
        // 50k > Nisab (8500), so payable
        assert!(result.is_payable);
        assert_eq!(result.net_assets, dec!(50000));
        assert_eq!(result.zakat_due, dec!(1250)); // 2.5% of 50k
    }

    #[test]
    fn test_penalty_withdrawal() {
        let config = ZakatConfig { gold_price_per_gram: Decimal::from(100), ..Default::default() };
        
        let fund = RestrictedFund::new_pension(20000.0, AccessibilityLevel::PenaltyWithdrawal)
            .penalty(2000.0); // 10% penalty
            
        let result = fund.calculate_zakat(&config).unwrap();
        // Net = 18000. Nisab = 8500. Payable.
        assert!(result.is_payable);
        assert_eq!(result.net_assets, dec!(18000));
        assert_eq!(result.zakat_due, dec!(450)); // 2.5% of 18k
    }
}
