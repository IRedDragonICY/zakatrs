use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the detailed breakdown of the Zakat calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZakatDetails {
    /// Total assets subject to Zakat calculation.
    pub total_assets: Decimal,
    /// Liabilities that can be deducted from the total assets.
    pub deductible_liabilities: Decimal,
    /// Net assets after deducting liabilities (total_assets - deductible_liabilities).
    pub net_assets: Decimal,
    /// The Nisab threshold applicable for this type of wealth.
    pub nisab_threshold: Decimal,
    /// Whether Zakat is due (net_assets >= nisab_threshold).
    pub is_payable: bool,
    /// The final Zakat amount due.
    pub zakat_due: Decimal,
    /// The type of wealth this calculation is for.
    pub wealth_type: WealthType,
}

impl ZakatDetails {
    pub fn new(
        total_assets: Decimal,
        deductible_liabilities: Decimal,
        nisab_threshold: Decimal,
        rate: Decimal,
        wealth_type: WealthType,
    ) -> Self {
        let net_assets = total_assets - deductible_liabilities;
        // Business rule: If net assets are negative, they are treated as zero for logic,
        // but it's good to preserve the actual value if needed.
        // For Nisab check: net_assets >= nisab_threshold
        let is_payable = net_assets >= nisab_threshold && net_assets > Decimal::ZERO;
        
        let zakat_due = if is_payable {
            net_assets * rate
        } else {
            Decimal::ZERO
        };

        ZakatDetails {
            total_assets,
            deductible_liabilities,
            net_assets,
            nisab_threshold,
            is_payable,
            zakat_due,
            wealth_type,
        }
    }
}

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZakatError {
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Missing configuration: {0}")]
    ConfigurationError(String),
}

/// Helper enum to categorize wealth types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WealthType {
    Fitrah,
    Gold,
    Silver,
    Business,
    Agriculture,
    Livestock,
    Income,
    Investment,
    Mining,
    Rikaz,
}
