//! Declarative macros for reducing boilerplate in Zakat asset definitions.
//!
//! The `zakat_asset!` macro generates common struct fields and their setters
//! that are shared across all Zakat asset types.

/// Macro for generating Zakat asset structs with common fields and methods.
///
/// This macro generates:
/// - The struct definition with user-defined fields plus common fields
///   (`liabilities_due_now`, `hawl_satisfied`, `label`, `id`, `_input_errors`)
/// - Common setters: `debt()`, `hawl()`, `label()`, `with_id()`
/// - A `validate()` method that returns deferred input errors
/// - Helper methods: `get_id()`, `get_label()`, `default_common()`
///
/// # Error Handling
///
/// Setters that require numeric conversion (like `debt()`) collect errors
/// instead of panicking. Call `validate()` or `calculate_zakat()` to surface errors.
///
/// # Usage
///
/// ```rust,ignore
/// crate::zakat_asset! {
///     #[derive(Debug, Clone, Serialize, Deserialize)]
///     pub struct MyAsset {
///         pub value: Decimal,
///         pub count: u32,
///     }
/// }
/// 
/// impl Default for MyAsset {
///     fn default() -> Self {
///         let (liabilities_due_now, hawl_satisfied, label, id, _input_errors) = Self::default_common();
///         Self {
///             value: Decimal::ZERO,
///             count: 0,
///             liabilities_due_now,
///             hawl_satisfied,
///             label,
///             id,
///             _input_errors,
///         }
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
            // User defined fields
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
            
            // === Common Fields (Standardized) ===
            /// Debts/liabilities that are due immediately and can be deducted.
            pub liabilities_due_now: rust_decimal::Decimal,
            /// Whether the Hawl (1 lunar year holding period) has been satisfied.
            pub hawl_satisfied: bool,
            /// Optional label for identifying this asset.
            pub label: Option<String>,
            /// Internal unique identifier.
            pub id: uuid::Uuid,
            /// Date when the asset was acquired (for precise Hawl calculation).
            pub acquisition_date: Option<chrono::NaiveDate>,
            /// Hidden field for deferred input validation errors.
            #[serde(skip)]
            _input_errors: Vec<$crate::types::ZakatError>,
        }

        impl $name {
            // Common Setters
            
            /// Creates a new instance with default values.
            pub fn new() -> Self { Self::default() }
            
            /// Sets deductible debt.
            pub fn debt(mut self, val: impl $crate::inputs::IntoZakatDecimal) -> Self {
                match val.into_zakat_decimal() {
                    Ok(v) => self.liabilities_due_now = v,
                    Err(e) => self._input_errors.push(e),
                }
                self
            }

            pub fn hawl(mut self, satisfied: bool) -> Self {
                self.hawl_satisfied = satisfied;
                self
            }

            pub fn label(mut self, val: impl Into<String>) -> Self {
                self.label = Some(val.into());
                self
            }

            pub fn acquired_on(mut self, date: chrono::NaiveDate) -> Self {
                self.acquisition_date = Some(date);
                self
            }

            pub fn with_id(mut self, id: uuid::Uuid) -> Self {
                self.id = id;
                self
            }
            
            /// Internal helper to init common fields.
            /// Returns (liabilities_due_now, hawl_satisfied, label, id, _input_errors, acquisition_date)
            fn default_common() -> (rust_decimal::Decimal, bool, Option<String>, uuid::Uuid, Vec<$crate::types::ZakatError>, Option<chrono::NaiveDate>) {
                (rust_decimal::Decimal::ZERO, true, None, uuid::Uuid::new_v4(), Vec::new(), None)
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
                    _ => Err($crate::types::ZakatError::MultipleErrors(self._input_errors.clone())),
                }
            }
            
            /// Returns the unique ID of the asset.
            pub fn get_id(&self) -> uuid::Uuid { self.id }
            
            /// Returns the optional label of the asset.
            pub fn get_label(&self) -> Option<String> { self.label.clone() }
        }
    };
}
