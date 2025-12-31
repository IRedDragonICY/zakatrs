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

/// Sanitizes a numeric string by removing common formatting characters.
/// 
/// This function handles:
/// - Currency symbols (`$`, `£`, `€`, `¥`) - removed
/// - Underscores (`_`) - Rust-style numeric separators, removed
/// - Commas (`,`) - intelligently handled:
///   - If comma is the last separator AND followed by 1-2 digits, treated as decimal (European format)
///   - Otherwise, treated as thousands separator (US/UK format)
/// - Leading/trailing whitespace - trimmed
///
/// # Examples
/// - `"$1,000.00"` → `"1000.00"` (US format)
/// - `"€12,50"` → `"12.50"` (European format)
/// - `"1.234,56"` → `"1234.56"` (European thousands + decimal)
///
/// Negative numbers and decimal points are preserved.
fn sanitize_numeric_string(s: &str) -> String {
    let mut result = s.trim().to_string();
    
    // Remove currency symbols and underscores
    result = result.replace(['$', '£', '€', '¥', '_'], "");
    
    // Heuristic for comma handling:
    // If comma is the last separator and followed by 1-2 digits at end,
    // treat as decimal point (European format like "12,50" or "1.234,56")
    if let Some(comma_pos) = result.rfind(',') {
        let after_comma = &result[comma_pos + 1..];
        let dot_pos = result.rfind('.');
        
        // European decimal: comma after any dot, or no dot and 1-2 digits after comma
        let is_european_decimal = (dot_pos.is_none() || comma_pos > dot_pos.unwrap())
            && after_comma.len() <= 2 
            && !after_comma.is_empty()
            && after_comma.chars().all(|c| c.is_ascii_digit());
        
        if is_european_decimal {
            // Replace this comma with dot, remove other commas and dots (thousands separators)
            let before_comma = result[..comma_pos].replace([',', '.'], "");
            result = format!("{}.{}", before_comma, after_comma);
        } else {
            // Comma is thousands separator - remove all commas
            result = result.replace(',', "");
        }
    }
    
    result
}

impl IntoZakatDecimal for &str {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let sanitized = sanitize_numeric_string(self);
        Decimal::from_str(&sanitized).map_err(|e| ZakatError::InvalidInput {
            field: "string".to_string(),
            value: self.to_string(),
            reason: format!("Parse error: {}", e),
            source_label: None,
        })
    }
}

impl IntoZakatDecimal for String {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let sanitized = sanitize_numeric_string(&self);
        Decimal::from_str(&sanitized).map_err(|e| ZakatError::InvalidInput {
            field: "string".to_string(),
            value: self.clone(),
            reason: format!("Parse error: {}", e),
            source_label: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_currency_with_comma() {
        let result = "$1,000.00".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1000.00").unwrap());
    }

    #[test]
    fn test_sanitize_underscores() {
        let result = "1_000".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1000").unwrap());
    }

    #[test]
    fn test_sanitize_whitespace() {
        let result = "  500 ".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("500").unwrap());
    }

    #[test]
    fn test_sanitize_negative_number() {
        let result = "-100.50".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("-100.50").unwrap());
    }

    #[test]
    fn test_sanitize_euro_currency() {
        let result = "€2,500.75".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("2500.75").unwrap());
    }

    #[test]
    fn test_sanitize_pound_with_underscores() {
        let result = "£1_234_567.89".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234567.89").unwrap());
    }

    #[test]
    fn test_sanitize_yen() {
        let result = "¥50000".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("50000").unwrap());
    }

    #[test]
    fn test_string_type_sanitization() {
        let input = String::from("$5,000.00");
        let result = input.into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("5000.00").unwrap());
    }

    // === European Locale Tests ===
    
    #[test]
    fn test_european_decimal_format() {
        // "€12,50" should become 12.50, not 1250
        let result = "€12,50".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("12.50").unwrap());
    }

    #[test]
    fn test_european_decimal_single_digit() {
        // "€12,5" should become 12.5
        let result = "€12,5".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("12.5").unwrap());
    }

    #[test]
    fn test_european_thousands_with_decimal() {
        // "1.234,56" (European thousands + decimal) should become 1234.56
        let result = "1.234,56".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.56").unwrap());
    }

    #[test]
    fn test_us_format_still_works() {
        // "1,234.56" (US format) should still work
        let result = "$1,234.56".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.56").unwrap());
    }

    #[test]
    fn test_large_european_format() {
        // "€1.234.567,89" should become 1234567.89
        let result = "€1.234.567,89".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234567.89").unwrap());
    }
}

