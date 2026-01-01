//! Declarative macros for reducing boilerplate in Zakat asset definitions.
//!
//! The `zakat_asset!` macro generates common struct fields and their setters
//! that are shared across all Zakat asset types.

/// Macro for generating Zakat asset structs with common fields and methods.
///
/// This macro generates:
/// - The struct definition with user-defined fields plus common fields
///   (`liabilities_due_now`, `hawl_satisfied`, `label`, `_input_errors`)
/// - A `new()` constructor
/// - Standard setters: `debt()`, `hawl()`, `label()`
/// - A `validate()` method that returns the first deferred input            /// - Implementation of `get_label()`, `is_valid()` and `validate_input()` for `CalculateZakat` trait
///
/// # Error Handling
///
/// Setters that require numeric conversion (like `debt()`) now collect errors
/// instead of panicking. Call `validate()` or `calculate_zakat()` to surface errors.
///
/// # Usage
///
/// ```rust,ignore
/// zakat_asset! {
///     /// Documentation for the struct
///     pub struct MyAsset {
///         pub value: Decimal,
///         pub count: u32,
///     }
/// }
/// ```
///
/// The user must still implement `calculate_zakat` manually as it differs per asset.
#[macro_export]
macro_rules! zakat_asset {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
            // === Common Fields (auto-generated) ===
            /// Debts/liabilities that are due immediately and can be deducted.
            pub liabilities_due_now: rust_decimal::Decimal,
            /// Whether the Hawl (1 lunar year holding period) has been satisfied.
            pub hawl_satisfied: bool,
            /// Optional label for identifying this asset in portfolio reports.
            pub label: Option<String>,
            // Internal unique identifier
            _id: uuid::Uuid,
            // Hidden field for deferred input validation errors
            _input_errors: Vec<$crate::types::ZakatError>,
        }

        impl $name {
            /// Creates a new instance with default values.
            pub fn new() -> Self {
                Self {
                    _id: uuid::Uuid::new_v4(),
                    _input_errors: Vec::new(),
                    ..Default::default()
                }
            }

            /// Sets the deductible debt/liabilities due now.
            /// 
            /// If the value cannot be converted to a valid decimal, the error is
            /// collected and will be returned by `validate()` or `calculate_zakat()`.
            pub fn debt(mut self, val: impl $crate::inputs::IntoZakatDecimal) -> Self {
                match val.into_zakat_decimal() {
                    Ok(v) => self.liabilities_due_now = v,
                    Err(e) => {
                        self._input_errors.push(e);
                        // Keep default (ZERO)
                    }
                }
                self
            }

            /// Sets whether the Hawl (1 lunar year) requirement is satisfied.
            pub fn hawl(mut self, satisfied: bool) -> Self {
                self.hawl_satisfied = satisfied;
                self
            }

            /// Sets an optional label for this asset.
            pub fn label(mut self, val: impl Into<String>) -> Self {
                self.label = Some(val.into());
                self
            }

            /// Validates the asset and returns any input errors.
            ///
            /// - If no errors, returns `Ok(())`.
            /// - If 1 error, returns `Err(that_error)`.
            /// - If >1 errors, returns `Err(ZakatError::MultipleErrors(...))`.
            pub fn validate(&self) -> Result<(), $crate::types::ZakatError> {
                match self._input_errors.len() {
                    0 => Ok(()),
                    1 => Err(self._input_errors[0].clone()),
                    _ => Err($crate::types::ZakatError::MultipleErrors(
                        self._input_errors.clone()
                    )),
                }
            }

            /// Returns the unique ID of the asset.
            pub fn get_id(&self) -> uuid::Uuid {
                self._id
            }

            /// Restores the asset ID (for database/serialization restoration).
            pub fn with_id(mut self, id: uuid::Uuid) -> Self {
                self._id = id;
                self
            }
        }
    };
}

