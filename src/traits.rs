
use crate::types::{ZakatDetails, ZakatError};

/// Trait to be implemented by all Zakat calculators.
pub trait CalculateZakat {
    /// Calculate Zakat details.
    ///
    /// * `Result<ZakatDetails, ZakatError>`
    fn calculate_zakat(&self, config: &crate::config::ZakatConfig) -> Result<ZakatDetails, ZakatError>;
}
