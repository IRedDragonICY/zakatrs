/// Macro for auto-generating Dart FFI wrappers from Zakat asset types.
///
/// This macro creates FRB-compatible wrapper structs with fluent builder APIs
/// that match the core Rust API pattern.

/// Generate a Dart-exportable wrapper for a Zakat asset type.
///
/// # Usage
///
/// ```ignore
/// dart_export_asset! {
///     /// Documentation for the wrapper
///     BusinessZakat as DartBusiness {
///         // Decimal fields (use FrbDecimal in Dart)
///         decimal: [cash, inventory, receivables, liabilities],
///         // Boolean fields  
///         bool: [hawl],
///         // String fields
///         string: [label],
///     }
/// }
/// ```
///
/// # With Extra Impl Block
///
/// You can inject additional methods (like static constructors) using `extra_impl`:
///
/// ```ignore
/// dart_export_asset! {
///     /// Precious metals wrapper
///     PreciousMetals as DartPreciousMetals {
///         decimal: [weight, debt],
///         bool: [hawl],
///         string: [label],
///         u32: [purity],
///     }
///     extra_impl: {
///         /// Create a new gold asset.
///         #[flutter_rust_bridge::frb(sync)]
///         pub fn gold(weight_grams: $crate::api::types::FrbDecimal) -> Self {
///             Self { inner: PreciousMetals::gold(weight_grams.value) }
///         }
///     }
/// }
/// ```
///
/// This generates:
/// - `DartBusiness` struct wrapping the core type
/// - `new()` constructor
/// - Fluent setters for each field (mutating in place for Opaque compat)
/// - `calculate(&DartZakatConfig) -> Result<DartZakatResult>` method
/// - Any additional methods from `extra_impl`
#[macro_export]
macro_rules! dart_export_asset {
    (
        $(#[$meta:meta])*
        $core_path:path as $dart_name:ident {
            $(decimal: [$($dec_field:ident),* $(,)?])?
            $(, bool: [$($bool_field:ident),* $(,)?])?
            $(, string: [$($str_field:ident),* $(,)?])?
            $(, u32: [$($u32_field:ident),* $(,)?])?
            $(,)?
        }
        $(extra_impl: { $($extra:tt)* })?
    ) => {
        $(#[$meta])*
        pub struct $dart_name {
            inner: $core_path,
        }

        impl $dart_name {
            /// Create a new instance with default values.
            #[flutter_rust_bridge::frb(sync)]
            pub fn new() -> Self {
                Self {
                    inner: <$core_path>::new(),
                }
            }

            // Generate setters for Decimal fields
            $($(
                #[doc = concat!("Set the `", stringify!($dec_field), "` field.")]
                #[flutter_rust_bridge::frb(sync)]
                #[allow(deprecated)] // May call deprecated methods for backward compat
                pub fn $dec_field(&mut self, value: $crate::api::types::FrbDecimal) {
                     let inner = std::mem::take(&mut self.inner);
                     self.inner = inner.$dec_field(value.value);
                }
            )*)?

            // Generate setters for bool fields  
            $($(
                #[doc = concat!("Set the `", stringify!($bool_field), "` field.")]
                #[flutter_rust_bridge::frb(sync)]
                pub fn $bool_field(&mut self, value: bool) {
                     let inner = std::mem::take(&mut self.inner);
                     self.inner = inner.$bool_field(value);
                }
            )*)?

            // Generate setters for String fields
            $($(
                #[doc = concat!("Set the `", stringify!($str_field), "` field.")]
                #[flutter_rust_bridge::frb(sync)]
                pub fn $str_field(&mut self, value: String) {
                     let inner = std::mem::take(&mut self.inner);
                     self.inner = inner.$str_field(value);
                }
            )*)?

            // Generate setters for u32 fields
            $($(
                #[doc = concat!("Set the `", stringify!($u32_field), "` field.")]
                #[flutter_rust_bridge::frb(sync)]
                pub fn $u32_field(&mut self, value: u32) {
                     let inner = std::mem::take(&mut self.inner);
                     self.inner = inner.$u32_field(value);
                }
            )*)?

            /// Calculate Zakat for this asset.
            #[flutter_rust_bridge::frb(sync)]
            pub fn calculate(&self, config: &$crate::api::types::DartZakatConfig) -> anyhow::Result<$crate::api::types::DartZakatResult> {
                use zakat::traits::CalculateZakat;
                
                let details = self.inner.calculate_zakat(&config.inner)
                    .map_err(|e| anyhow::anyhow!("Calculation failed: {:?}", e))?;
                
                Ok($crate::api::types::DartZakatResult::from_core(details))
            }
            
            /// Get the asset ID.
            #[flutter_rust_bridge::frb(sync)]
            pub fn get_id(&self) -> String {
                self.inner.get_id().to_string()
            }
            
            /// Get the asset label.
            #[flutter_rust_bridge::frb(sync)]
            pub fn get_label(&self) -> Option<String> {
                self.inner.get_label()
            }
            
            // Inject any extra implementation provided by the caller
            $($($extra)*)?
        }
        
        impl Default for $dart_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
