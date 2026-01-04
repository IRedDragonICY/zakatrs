//! Validation utilities for Zakat assets.
//! 
//! Extracted to reduce boilerplate in calculate_zakat implementations.

use rust_decimal::Decimal;
use crate::types::{ZakatError, InvalidInputDetails};

pub struct Validator;

impl Validator {
    /// Checks a list of (field_name, value) pairs to ensure they are >= 0.
    /// 
    /// # Arguments
    /// * `checks`: A slice of tuples `("field_name", decimal_value)`.
    /// * `source_label`: Optional label for error context.
    pub fn ensure_non_negative(
        checks: &[(&str, Decimal)], 
        source_label: Option<String>
    ) -> Result<(), ZakatError> {
        for (field, val) in checks {
            if *val < Decimal::ZERO {
                return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                    field: field.to_string(),
                    value: "negative".to_string(),
                    reason_key: "error-negative-value".to_string(),
                    source_label: source_label.clone(),
                    suggestion: Some("Zakat inputs must be positive. Did you mean to subtract this value manually?".to_string()),
                    ..Default::default()
                })));
            }
        }
        Ok(())
    }

    /// Ensures a required Option field is present.
    pub fn require<'a, T>(
        val: &'a Option<T>,
        field: &str,
        source_label: Option<String>
    ) -> Result<&'a T, ZakatError> {
        val.as_ref().ok_or_else(|| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: field.to_string(),
            value: "None".to_string(),
            reason_key: "error-type-required".to_string(),
            source_label,
            suggestion: Some("This field is required and cannot be empty.".to_string()),
            ..Default::default()
        })))
    }
}
