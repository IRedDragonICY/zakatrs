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
/// This generates:
/// - `DartBusiness` struct wrapping the core type
/// - `new()` constructor
/// - Fluent setters for each field (mutating in place for Opaque compat)
/// - `calculate(&DartZakatConfig) -> Result<DartZakatResult>` method
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
        }
        
        impl Default for $dart_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

/// Generate a Dart-exportable wrapper for PreciousMetals with metal type constructors.
#[macro_export]
macro_rules! dart_export_precious_metals {
    () => {
        /// Precious metals (gold/silver) asset wrapper for Dart.
        pub struct DartPreciousMetals {
            inner: zakat::maal::precious_metals::PreciousMetals,
        }

        impl DartPreciousMetals {
            /// Create a new gold asset.
            #[flutter_rust_bridge::frb(sync)]
            pub fn gold(weight_grams: $crate::api::types::FrbDecimal) -> Self {
                Self {
                    inner: zakat::maal::precious_metals::PreciousMetals::gold(weight_grams.value),
                }
            }
            
            /// Create a new silver asset.
            #[flutter_rust_bridge::frb(sync)]
            pub fn silver(weight_grams: $crate::api::types::FrbDecimal) -> Self {
                Self {
                    inner: zakat::maal::precious_metals::PreciousMetals::silver(weight_grams.value),
                }
            }
            
            /// Set the weight in grams.
            #[flutter_rust_bridge::frb(sync)]
            pub fn weight(&mut self, grams: $crate::api::types::FrbDecimal) {
                let inner = std::mem::take(&mut self.inner);
                self.inner = inner.weight(grams.value);
            }
            
            /// Set the purity (karat for gold 1-24, or per-mille for silver 1-1000).
            #[flutter_rust_bridge::frb(sync)]
            pub fn purity(&mut self, value: u32) {
                let inner = std::mem::take(&mut self.inner);
                self.inner = inner.purity(value);
            }
            
            /// Set whether Hawl (1 lunar year) is satisfied.
            #[flutter_rust_bridge::frb(sync)]
            pub fn hawl(&mut self, satisfied: bool) {
                let inner = std::mem::take(&mut self.inner);
                self.inner = inner.hawl(satisfied);
            }
            
            /// Set the asset label.
            #[flutter_rust_bridge::frb(sync)]
            pub fn label(&mut self, label: String) {
                let inner = std::mem::take(&mut self.inner);
                self.inner = inner.label(label);
            }
            
            /// Set liabilities to deduct.
            #[flutter_rust_bridge::frb(sync)]
            pub fn debt(&mut self, amount: $crate::api::types::FrbDecimal) {
                let inner = std::mem::take(&mut self.inner);
                self.inner = inner.debt(amount.value);
            }

            /// Calculate Zakat for this precious metal.
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
        }

        impl Default for DartPreciousMetals {
            fn default() -> Self {
                // Default precious metals isn't directly constructible via new(), 
                // but PreciousMetals implements Default (zero value gold).
                Self {
                    inner: zakat::maal::precious_metals::PreciousMetals::default(),
                }
            }
        }
    };
}
