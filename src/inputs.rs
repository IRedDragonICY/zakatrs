use rust_decimal::Decimal;
use std::str::FromStr;
use crate::types::ZakatError;

/// Trait for converting various types into `Decimal` for Zakat calculations.
/// 
/// This trait allows users to pass `i32`, `f64`, `&str`, etc. directly into
/// constructors without needing to wrap them in `Decimal` conversion methods.
pub trait IntoZakatDecimal {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError>;
}

// Implement for Decimal (passthrough)
impl IntoZakatDecimal for Decimal {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        Ok(self)
    }
}

// Implement for Integers
macro_rules! impl_into_zakat_decimal_int {
    ($($t:ty),*) => {
        $(
            impl IntoZakatDecimal for $t {
                fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
                    Ok(Decimal::from(self))
                }
            }
        )*
    };
}

impl_into_zakat_decimal_int!(i32, u32, i64, u64, isize, usize);

// Implement for Floats
macro_rules! impl_into_zakat_decimal_float {
    ($($t:ty),*) => {
        $(
            impl IntoZakatDecimal for $t {
                fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
                     // Use string formatting to avoid binary precision noise.
                     // This aligns with user expectations for simple decimals like 0.025.
                    let s = self.to_string();
                    Decimal::from_str(&s).map_err(|_| ZakatError::InvalidInput {
                        field: "fractional".to_string(),
                        value: s,
                        reason: "Invalid float value".to_string(),
                        source_label: None,
                    })
                }
            }
        )*
    };
}

impl_into_zakat_decimal_float!(f32, f64);

// Implement for Strings
impl IntoZakatDecimal for &str {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let trimmed = self.trim();
        Decimal::from_str(trimmed).map_err(|e| ZakatError::InvalidInput {
            field: "string".to_string(),
            value: trimmed.to_string(),
            reason: format!("Parse error: {}", e),
            source_label: None,
        })
    }
}

impl IntoZakatDecimal for String {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let trimmed = self.trim();
        Decimal::from_str(trimmed).map_err(|e| ZakatError::InvalidInput {
            field: "string".to_string(),
            value: trimmed.to_string(),
            reason: format!("Parse error: {}", e),
            source_label: None,
        })
    }
}
