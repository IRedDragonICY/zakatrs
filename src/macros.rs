//! Declarative macros for reducing boilerplate in Zakat asset definitions.
//!
//! The `zakat_asset!` macro generates common struct fields and their setters
//! that are shared across all Zakat asset types.

/// Macro for generating Zakat asset structs with common fields and methods.
///
/// This macro generates:
/// - The struct definition with user-defined fields plus common fields
///   (`liabilities_due_now`, `hawl_satisfied`, `label`)
/// - A `new()` constructor
/// - Standard setters: `debt()`, `hawl()`, `label()`
/// - Implementation of `get_label()` for `CalculateZakat` trait
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
        }

        impl $name {
            /// Creates a new instance with default values.
            pub fn new() -> Self {
                Self::default()
            }

            /// Sets the deductible debt/liabilities due now.
            pub fn debt(mut self, val: impl $crate::inputs::IntoZakatDecimal) -> Self {
                if let Ok(v) = val.into_zakat_decimal() {
                    self.liabilities_due_now = v;
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
        }
    };
}
