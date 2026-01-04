use rust_decimal::Decimal;
use std::borrow::Cow;
use std::str::FromStr;
use crate::types::{ZakatError, InvalidInputDetails};

const MAX_INPUT_LEN: usize = 64;

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
                     Decimal::from_str(&s).map_err(|_| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                        field: "fractional".to_string(),
                        value: s,
                        reason_key: "error-invalid-float".to_string(),
                        args: None,
                        source_label: None,
                        asset_id: None,
                        suggestion: Some("Ensure you are using a valid number format (e.g., 1000.50).".to_string()),
                    })))
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
/// - Arabic numerals: Eastern Arabic (`٠` through `٩`) and Perso-Arabic (`۰` through `۹`) normalized to ASCII
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
/// - `"١٢٣٤.٥٠"` → `"1234.50"` (Eastern Arabic numerals)
///
/// Negative numbers and decimal points are preserved.
/// Validates if a string can be parsed into a Zakat Decimal.
///
/// This is useful for UI layers (Python, Dart, JS) to "Try Parse" before sending data.
pub fn validate_numeric_format(s: &str) -> bool {
    let sanitized = match sanitize_numeric_string(s) {
        Ok(s) => s,
        Err(_) => return false,
    };
    Decimal::from_str(&sanitized).is_ok()
}

/// Sanitizes a numeric string by removing common formatting characters.
/// 
/// This function handles:
/// - Arabic numerals: Eastern Arabic (`٠` through `٩`) and Perso-Arabic (`۰` through `۹`) normalized to ASCII
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
/// - `"١٢٣٤.٥٠"` → `"1234.50"` (Eastern Arabic numerals)
///
/// # Performance
/// Returns `Cow::Borrowed` if the input is already clean (no sanitization needed),
/// avoiding heap allocation. Returns `Cow::Owned` only when sanitization is required.
///
/// Negative numbers and decimal points are preserved.
pub fn sanitize_numeric_string(s: &str) -> Result<Cow<'_, str>, ZakatError> {
    if s.len() > MAX_INPUT_LEN {
        return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: "input".to_string(),
            value: format!("{}...", &s[..std::cmp::min(s.len(), 20)]),
            reason_key: "error-input-too-long".to_string(),
            args: Some(std::collections::HashMap::from([("max".to_string(), MAX_INPUT_LEN.to_string())])),
            source_label: None,
            asset_id: None,
            suggestion: Some(format!("Maximum input length is {} characters.", MAX_INPUT_LEN)),
        })));
    }

    let trimmed = s.trim();
    
    // Fast path: Check if the string is already clean (only ASCII digits, '.', '-')
    // If so, return borrowed reference without any allocation
    let needs_sanitization = trimmed.chars().any(|c| {
        !matches!(c, '0'..='9' | '.' | '-')
    });
    
    if !needs_sanitization {
        return Ok(Cow::Borrowed(trimmed));
    }

    // Slow path: String needs sanitization, allocate and clean
    let mut buffer = String::with_capacity(trimmed.len());
    let mut last_comma_index = None;
    let mut last_dot_index = None;
    
    // Single pass for cleaning and normalization
    for c in trimmed.chars() {
        match c {
            // Arabic Numerals -> Normalize to ASCII
            '\u{0660}'..='\u{0669}' => buffer.push(char::from_u32(c as u32 - 0x0660 + '0' as u32).unwrap_or(c)),
            '\u{06F0}'..='\u{06F9}' => buffer.push(char::from_u32(c as u32 - 0x06F0 + '0' as u32).unwrap_or(c)),
            
            // Allowed characters
            '0'..='9' | '-' | '.' => {
                if c == '.' {
                    last_dot_index = Some(buffer.len());
                }
                buffer.push(c);
            },
            
            // Commas need tracking for heuristic
            ',' => {
                last_comma_index = Some(buffer.len());
                buffer.push(c);
            },

            // Eastern Arabic Decimal Separator (٫) -> Convert to dot
            '٫' => {
                 last_dot_index = Some(buffer.len());
                 buffer.push('.');
            },
            
            // Ignored characters (Currency, Spaces, Controls, Underscores, Arabic Thousands Sep)
            '$' | '£' | '€' | '¥' | '_' | ' ' | '\u{00A0}' | '٬' => {},
            
            // Skip control characters
            c if c.is_control() => {},
            
            // Keep others for compatibility (will cause parse error later if invalid)
            _ => buffer.push(c), 
        }
    }

    // Heuristic for comma handling (Post-processing on the cleaned buffer)
    if let Some(comma_pos) = last_comma_index {
        let len = buffer.len();
        let after_comma_len = len - 1 - comma_pos;
        
        // Check if it looks like a European decimal
        let is_european_decimal = (last_dot_index.is_none() || comma_pos > last_dot_index.unwrap())
            && after_comma_len > 0
            && after_comma_len <= 2
            && buffer[comma_pos+1..].chars().all(|c| c.is_ascii_digit());
            
        if is_european_decimal {
            // It's a decimal separator - convert comma to dot, remove dots (thousands seps)
            let mut final_res = String::with_capacity(buffer.len());
            for (i, c) in buffer.chars().enumerate() {
                if i == comma_pos {
                    final_res.push('.');
                } else if c == '.' {
                    // Skip (it was a thousands separator)
                } else if c == ',' {
                    // Skip (other commas)
                } else {
                    final_res.push(c);
                }
            }
            return Ok(Cow::Owned(final_res));
        } else {
            // Comma is a thousands separator (US). Remove all commas.
            buffer.retain(|c| c != ',');
        }
    }
    
    Ok(Cow::Owned(buffer))
}

/// Normalizes Arabic numerals to ASCII digits.
/// 
/// Handles:
/// - Eastern Arabic numerals: ٠١٢٣٤٥٦٧٨٩ (U+0660..U+0669)
/// - Perso-Arabic numerals: ۰۱۲۳۴۵۶۷۸۹ (U+06F0..U+06F9)
fn normalize_arabic_numerals(s: &str) -> String {
    // Kept for backward compatibility if used elsewhere, 
    // but `sanitize_numeric_string` now handles this inline.
    s.chars().map(|c| {
        match c {
            // Eastern Arabic numerals (٠-٩)
            '\u{0660}'..='\u{0669}' => char::from_u32(c as u32 - 0x0660 + '0' as u32).unwrap_or(c),
            // Perso-Arabic numerals (۰-۹)
            '\u{06F0}'..='\u{06F9}' => char::from_u32(c as u32 - 0x06F0 + '0' as u32).unwrap_or(c),
            _ => c,
        }
    }).collect()
}

impl IntoZakatDecimal for &str {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let sanitized = sanitize_numeric_string(self)?;
        Decimal::from_str(&sanitized).map_err(|e| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: "string".to_string(),
            value: self.to_string(),
            reason_key: "error-parse-error".to_string(),
            args: Some(std::collections::HashMap::from([("details".to_string(), e.to_string())])),
            source_label: None,
            asset_id: None,
            suggestion: Some("Ensure you are using a valid number format (e.g., 1000.50). Remove symbols like '$' if present.".to_string()),
        })))
    }
}

impl IntoZakatDecimal for String {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        let sanitized = sanitize_numeric_string(&self)?;
        Decimal::from_str(&sanitized).map_err(|e| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: "string".to_string(),
            value: self.clone(),
            reason_key: "error-parse-error".to_string(),
            args: Some(std::collections::HashMap::from([("details".to_string(), e.to_string())])),
            source_label: None,
            asset_id: None,
            suggestion: Some("Ensure you are using a valid number format (e.g., 1000.50). Remove symbols like '$' if present.".to_string()),
        })))
    }
}

/// Locale specification for unambiguous numeric input parsing.
///
/// The heuristic-based parsing in `IntoZakatDecimal` for `&str` can be ambiguous
/// (e.g., "1,234" could be US thousands or EU decimal). Use `LocalizedInput` with
/// an explicit locale when you need guaranteed correct parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputLocale {
    /// US/UK format: comma = thousands, dot = decimal (e.g., "1,000.00")
    US,
    /// European format: dot = thousands, comma = decimal (e.g., "1.000,00")
    EU,
    /// Eastern Arabic format: Arabic-Indic numerals with Arabic decimal separator
    EasternArabic,
}

/// A string input with an explicit locale for unambiguous parsing.
///
/// This struct is used with the `with_locale` helper function to specify
/// the exact format of a numeric string.
///
/// # Example
/// ```rust,ignore
/// use zakat::{with_locale, InputLocale};
///
/// // European format: "1.234,50" means 1234.50
/// let value = with_locale("1.234,50", InputLocale::EU);
/// let decimal = value.into_zakat_decimal().unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct LocalizedInput<'a> {
    value: &'a str,
    locale: InputLocale,
}

/// Creates a `LocalizedInput` for explicit locale-aware parsing.
///
/// # Arguments
/// * `val` - The numeric string to parse.
/// * `locale` - The locale specifying the number format.
///
/// # Example
/// ```rust,ignore
/// use zakat::{with_locale, InputLocale};
///
/// // US format: "1,234.50" means 1234.50
/// let decimal = with_locale("$1,234.50", InputLocale::US).into_zakat_decimal().unwrap();
///
/// // EU format: "1.234,50" means 1234.50
/// let decimal = with_locale("€1.234,50", InputLocale::EU).into_zakat_decimal().unwrap();
/// ```
pub fn with_locale(val: &str, locale: InputLocale) -> LocalizedInput<'_> {
    LocalizedInput { value: val, locale }
}

impl IntoZakatDecimal for LocalizedInput<'_> {
    fn into_zakat_decimal(self) -> Result<Decimal, ZakatError> {
        if self.value.len() > MAX_INPUT_LEN {
            return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "localized_input".to_string(),
                value: format!("{}...", &self.value[..std::cmp::min(self.value.len(), 20)]),
                reason_key: "error-input-too-long".to_string(),
                args: Some(std::collections::HashMap::from([("max".to_string(), MAX_INPUT_LEN.to_string())])),
                source_label: None,
                asset_id: None,
                suggestion: Some(format!("Maximum input length is {} characters.", MAX_INPUT_LEN)),
            })));
        }

        // First normalize Arabic numerals
        let normalized = normalize_arabic_numerals(self.value);
        let mut result = normalized.trim().to_string();
        
        // Remove currency symbols and underscores
        result = result.replace(['$', '£', '€', '¥', '_'], "");
        
        match self.locale {
            InputLocale::US => {
                // US format: comma = thousands separator, dot = decimal
                result = result.replace(',', "");
                // Dot remains as decimal separator
            }
            InputLocale::EU => {
                // EU format: dot = thousands separator, comma = decimal
                result = result.replace('.', "");  // Remove thousands separators
                result = result.replace(',', "."); // Convert comma decimal to dot
            }
            InputLocale::EasternArabic => {
                // Arabic decimal separator (٫) replaced with dot
                result = result.replace('٫', ".");
                // Remove Arabic thousands separator (٬)
                result = result.replace('٬', "");
                // Also handle comma/dot if mixed with Arabic numerals
                result = result.replace(',', "");
            }
        }
        
        Decimal::from_str(&result).map_err(|e| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: "localized_input".to_string(),
            value: self.value.to_string(),
            reason_key: "error-parse-locale".to_string(),
            args: Some(std::collections::HashMap::from([
                ("locale".to_string(), format!("{:?}", self.locale)), 
                ("details".to_string(), e.to_string())
            ])),
            source_label: None,
            asset_id: None,
            suggestion: Some("Ensure you are using a valid number format (e.g., 1000.50). Remove symbols like '$' if present.".to_string()),
        })))
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

    // === Arabic Numeral Tests ===

    #[test]
    fn test_eastern_arabic_numerals() {
        // Eastern Arabic: "١٢٣٤.٥٠" should become 1234.50
        let result = "١٢٣٤.٥٠".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.50").unwrap());
    }

    #[test]
    fn test_perso_arabic_numerals() {
        // Perso-Arabic: "۱۲۳۴.۵۰" should become 1234.50
        let result = "۱۲۳۴.۵۰".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.50").unwrap());
    }

    #[test]
    fn test_arabic_with_currency() {
        // Mixed: Eastern Arabic with common formatting
        let result = "١,٠٠٠.٥٠".into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1000.50").unwrap());
    }

    // === Explicit LocalizedInput Tests ===
    
    #[test]
    fn test_localized_input_us() {
        // US format with explicit locale
        let result = with_locale("$1,234.56", InputLocale::US).into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.56").unwrap());
    }

    #[test]
    fn test_localized_input_eu() {
        // EU format: dot = thousands, comma = decimal
        let result = with_locale("€1.234,56", InputLocale::EU).into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.56").unwrap());
    }

    #[test]
    fn test_localized_input_eu_no_thousands() {
        // EU format without thousands separator
        let result = with_locale("€12,50", InputLocale::EU).into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("12.50").unwrap());
    }

    #[test]
    fn test_localized_input_eastern_arabic() {
        // Eastern Arabic numerals with Arabic decimal separator
        let result = with_locale("١٢٣٤.٥٠", InputLocale::EasternArabic).into_zakat_decimal().unwrap();
        assert_eq!(result, Decimal::from_str("1234.50").unwrap());
    }
}


// --- FFI Utilities ---

/// Trait to convert types to String for FFI boundaries (WASM/UniFFI).
/// Ensures Option<T> handles None gracefully.
pub trait ToFfiString {
    fn to_ffi_string(&self) -> String;
}

// Convert Option types
impl<T: ToFfiString> ToFfiString for Option<T> {
    fn to_ffi_string(&self) -> String {
        match self {
            Some(v) => v.to_ffi_string(),
            None => String::new(),
        }
    }
}

// Specific Implementations
impl ToFfiString for String {
    fn to_ffi_string(&self) -> String { self.clone() }
}
impl ToFfiString for &str {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl ToFfiString for bool {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl ToFfiString for rust_decimal::Decimal {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
impl ToFfiString for uuid::Uuid {
    fn to_ffi_string(&self) -> String { self.to_string() }
}
// Integers
impl ToFfiString for i32 { fn to_ffi_string(&self) -> String { self.to_string() } }
impl ToFfiString for u32 { fn to_ffi_string(&self) -> String { self.to_string() } }
impl ToFfiString for i64 { fn to_ffi_string(&self) -> String { self.to_string() } }
impl ToFfiString for u64 { fn to_ffi_string(&self) -> String { self.to_string() } }
impl ToFfiString for isize { fn to_ffi_string(&self) -> String { self.to_string() } }
impl ToFfiString for usize { fn to_ffi_string(&self) -> String { self.to_string() } }


/// Trait to parse types from String for FFI boundaries.
/// Handles Option<T> by treating empty strings/null as None.
pub trait FromFfiString: Sized {
    type Err;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err>;
}

// Blanket impl for Option
impl<T: FromFfiString> FromFfiString for Option<T> {
    type Err = T::Err;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() || s.eq_ignore_ascii_case("none") || s.eq_ignore_ascii_case("null") {
            Ok(None)
        } else {
            T::from_ffi_string(s).map(Some)
        }
    }
}

// Specific Implementations
impl FromFfiString for rust_decimal::Decimal {
    type Err = rust_decimal::Error;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        rust_decimal::Decimal::from_str(s)
    }
}

impl FromFfiString for bool {
    type Err = std::str::ParseBoolError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
         // enhanced bool parsing
         match s.to_lowercase().as_str() {
             "1" | "true" | "yes" | "on" => Ok(true),
             "0" | "false" | "no" | "off" => Ok(false),
             _ => std::str::FromStr::from_str(s),
         }
    }
}

impl FromFfiString for String {
    type Err = std::convert::Infallible;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string())
    }
}

impl FromFfiString for uuid::Uuid {
    type Err = uuid::Error;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s)
    }
}

// Integers
impl FromFfiString for i32 { type Err = std::num::ParseIntError; fn from_ffi_string(s: &str) -> Result<Self, Self::Err> { s.parse() } }
impl FromFfiString for u32 { type Err = std::num::ParseIntError; fn from_ffi_string(s: &str) -> Result<Self, Self::Err> { s.parse() } }
impl FromFfiString for i64 { type Err = std::num::ParseIntError; fn from_ffi_string(s: &str) -> Result<Self, Self::Err> { s.parse() } }
impl FromFfiString for u64 { type Err = std::num::ParseIntError; fn from_ffi_string(s: &str) -> Result<Self, Self::Err> { s.parse() } }
