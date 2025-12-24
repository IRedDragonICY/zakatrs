use rust_decimal::Decimal;
use crate::types::{ZakatDetails, ZakatError};

/// Trait to be implemented by all Zakat calculators.
pub trait CalculateZakat {
    /// Calculate Zakat details.
    ///
    /// # Arguments
    ///
    /// * `debts` - Deductible liabilities (if applicable for the wealth type).
    ///
    /// # Returns
    ///
    /// * `Result<ZakatDetails, ZakatError>`
    fn calculate_zakat(&self, debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError>;
}
