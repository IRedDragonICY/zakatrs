use std::sync::Arc;
use std::str::FromStr;
use crate::prelude::*;
use rust_decimal::Decimal;


/// UniFFI-compatible error type for Kotlin bindings.
#[derive(Debug, uniffi::Error)]
#[uniffi(flat_error)]
pub enum KotlinZakatError {
    /// Failed to parse a decimal value from string input.
    ParseError { field: String, message: String },
    /// Error during Zakat calculation.
    CalculationError { reason: String },
}

impl std::fmt::Display for KotlinZakatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { field, message } => {
                write!(f, "Failed to parse '{}': {}", field, message)
            }
            Self::CalculationError { reason } => {
                write!(f, "Calculation error: {}", reason)
            }
        }
    }
}

impl std::error::Error for KotlinZakatError {}

/// Helper function to parse a String to Decimal, returning a KotlinZakatError on failure.
fn parse_decimal(field: &str, value: &str) -> Result<Decimal, KotlinZakatError> {
    Decimal::from_str(value).map_err(|e| KotlinZakatError::ParseError {
        field: field.to_string(),
        message: format!("Invalid decimal '{}': {}", value, e),
    })
}

// --- Facade: Configuration ---
#[derive(uniffi::Record)]
pub struct KotlinZakatConfig {
    pub gold_price: String,
    pub silver_price: String,
}

#[derive(uniffi::Object)]
pub struct KotlinConfigWrapper {
    pub inner: ZakatConfig,
}

#[uniffi::export]
impl KotlinConfigWrapper {
    #[uniffi::constructor]
    pub fn new(gold_price: String, silver_price: String) -> Result<Arc<Self>, KotlinZakatError> {
        let gold = parse_decimal("gold_price", &gold_price)?;
        let silver = parse_decimal("silver_price", &silver_price)?;
        
        let cfg = ZakatConfig {
            gold_price_per_gram: gold,
            silver_price_per_gram: silver,
            ..Default::default()
        };
        Ok(Arc::new(Self { inner: cfg }))
    }
}

// --- Facade: Assets (Removed in favor of zakat_ffi_export! generated bindings) ---
// See src/maal/business.rs -> uniffi_ffi
// See src/maal/precious_metals.rs -> uniffi_ffi

