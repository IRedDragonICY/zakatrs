use rust_decimal::Decimal;
use std::str::FromStr;
use crate::types::ZakatError;

/// Trait for converting various types into `Decimal` for Zakat calculations.
/// 
/// This trait allows users to pass `i32`, `f64`, `&str`, etc. directly into
/// constructors without needing to wrap them in `dec!()` or `Decimal::from()`.
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
                    Decimal::from_f64_retain(self as f64)
                        .ok_or_else(|| ZakatError::InvalidInput(format!("Invalid float value: {}", self), None))
                }
            }
        )*
    };
}

impl_into_zakat_decimal_float!(f32, f64);

// Implement for Strings
impl IntoZakatDecimal for &str {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        Decimal::from_str(self).map_err(|e| ZakatError::InvalidInput(format!("Invalid string format: {}", e), None))
    }
}

impl IntoZakatDecimal for String {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        Decimal::from_str(&self).map_err(|e| ZakatError::InvalidInput(format!("Invalid string format: {}", e), None))
    }
}
