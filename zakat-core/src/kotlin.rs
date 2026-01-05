//! UniFFI Kotlin/Swift binding support types.
//!
//! This module provides the FFI layer for ZakatRS, exposing strict types
//! and methods for mobile consumption.
//!
//! The `uniffi` feature must be enabled for this module to be active.

use std::sync::Arc;
use crate::config::ZakatConfig;
use crate::types::{ZakatError, FfiZakatError};
use rust_decimal::Decimal;
use std::str::FromStr;

/// UniFFI-compatible error type for Kotlin/Swift bindings.
/// Wraps the generic `FfiZakatError` to provide a throwable exception.
#[derive(Debug, uniffi::Error)]
pub enum UniFFIZakatError {
    /// A generic Zakat error containing structured details.
    Generic {
        code: String,
        message: String,
        field: Option<String>,
        hint: Option<String>
    }
}

impl std::fmt::Display for UniFFIZakatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UniFFIZakatError::Generic { message, .. } => write!(f, "{}", message),
        }
    }
}

impl From<ZakatError> for UniFFIZakatError {
    fn from(e: ZakatError) -> Self {
        let ffi_err = FfiZakatError::from(e);
        UniFFIZakatError::Generic {
            code: ffi_err.code,
            message: ffi_err.message,
            field: ffi_err.field,
            hint: ffi_err.hint,
        }
    }
}

// Export ZakatConfig methods for FFI
#[uniffi::export]
impl ZakatConfig {
    #[uniffi::constructor]
    pub fn default_config() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Creates a pre-configured ZakatConfig for the Hanafi Madhab.
    #[uniffi::constructor]
    pub fn hanafi_factory(gold_price: String, silver_price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let gold = Decimal::from_str(&gold_price).map_err(|e| UniFFIZakatError::Generic{
            code: "VAL_ERR".into(), message: e.to_string(), field: Some("gold_price".into()), hint: None
        })?;
        let silver = Decimal::from_str(&silver_price).map_err(|e| UniFFIZakatError::Generic{
            code: "VAL_ERR".into(), message: e.to_string(), field: Some("silver_price".into()), hint: None
        })?;
        Ok(Arc::new(Self::hanafi(gold, silver)))
    }

    /// Creates a pre-configured ZakatConfig for the Shafi Madhab.
    #[uniffi::constructor]
    pub fn shafi_factory(gold_price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let gold = Decimal::from_str(&gold_price).map_err(|e| UniFFIZakatError::Generic{
            code: "VAL_ERR".into(), message: e.to_string(), field: Some("gold_price".into()), hint: None
        })?;
        Ok(Arc::new(Self::shafi(gold)))
    }

    /// Creates a pre-configured ZakatConfig for the Maliki Madhab.
    #[uniffi::constructor]
    pub fn maliki_factory(gold_price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let gold = Decimal::from_str(&gold_price).map_err(|e| UniFFIZakatError::Generic{
            code: "VAL_ERR".into(), message: e.to_string(), field: Some("gold_price".into()), hint: None
        })?;
        Ok(Arc::new(Self::maliki(gold)))
    }

    /// Creates a pre-configured ZakatConfig for the Hanbali Madhab.
    #[uniffi::constructor]
    pub fn hanbali_factory(gold_price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let gold = Decimal::from_str(&gold_price).map_err(|e| UniFFIZakatError::Generic{
            code: "VAL_ERR".into(), message: e.to_string(), field: Some("gold_price".into()), hint: None
        })?;
        Ok(Arc::new(Self::hanbali(gold)))
    }

    /// Initializes a `ZakatConfig` with regional Fiqh defaults.
    #[uniffi::constructor]
    pub fn for_region_factory(iso_code: String) -> Arc<Self> {
        Arc::new(Self::for_region(&iso_code))
    }

    pub fn set_gold_price(&self, price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let p = Decimal::from_str(&price).map_err(|e| UniFFIZakatError::Generic{code:"VAL".into(), message:e.to_string(), field:None, hint:None})?;
        Ok(Arc::new(self.clone().with_gold_price(p)))
    }

    pub fn set_silver_price(&self, price: String) -> Result<Arc<Self>, UniFFIZakatError> {
        let p = Decimal::from_str(&price).map_err(|e| UniFFIZakatError::Generic{code:"VAL".into(), message:e.to_string(), field:None, hint:None})?;
        Ok(Arc::new(self.clone().with_silver_price(p)))
    }
    
    pub fn set_gold_nisab(&self, grams: String) -> Result<Arc<Self>, UniFFIZakatError> {
         let p = Decimal::from_str(&grams).map_err(|e| UniFFIZakatError::Generic{code:"VAL".into(), message:e.to_string(), field:None, hint:None})?;
        Ok(Arc::new(self.clone().with_gold_nisab(p)))
    }
    
    pub fn set_silver_nisab(&self, grams: String) -> Result<Arc<Self>, UniFFIZakatError> {
         let p = Decimal::from_str(&grams).map_err(|e| UniFFIZakatError::Generic{code:"VAL".into(), message:e.to_string(), field:None, hint:None})?;
        Ok(Arc::new(self.clone().with_silver_nisab(p)))
    }

    pub fn set_locale_code(&self, locale: String) -> Arc<Self> {
        Arc::new(self.clone().with_locale_code(locale))
    }

    pub fn set_currency_code(&self, code: String) -> Arc<Self> {
        Arc::new(self.clone().with_currency_code(code))
    }

    
    // Getters for properties
    pub fn get_gold_price(&self) -> String {
        self.gold_price_per_gram.to_string()
    }

    pub fn get_silver_price(&self) -> String {
        self.silver_price_per_gram.to_string()
    }
    
    pub fn check_validity(&self) -> Result<(), UniFFIZakatError> {
        // map_err uses the From impl
        self.validate_internal().map_err(Into::into)
    }

    // Internal helper to avoid name collision with standard validate if exposed
    // But `validate` is already a method on ZakatConfig.
    // In Rust it returns Result<(), ZakatError>.
    // In FFI we want Result<(), UniFFIZakatError>.
    // We can't shadow the Rust method in the same impl block easily if checking?
    // Wait, this is a separate impl block. `self.validate()` calls the inherent method.
    // This function `validate` is exposed to FFI.
}

// Expose internal validate method call
trait InternalValidate {
    fn validate_internal(&self) -> Result<(), ZakatError>;
}
impl InternalValidate for ZakatConfig {
    fn validate_internal(&self) -> Result<(), ZakatError> {
        self.validate()
    }
}
