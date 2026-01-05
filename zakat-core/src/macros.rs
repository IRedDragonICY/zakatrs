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
        #[derive(schemars::JsonSchema)]
        #[serde(rename_all = "camelCase")]
        #[serde(default)]
        #[allow(deprecated)] // Internal usage of deprecated `liabilities_due_now` for backward compat
        $vis struct $name {
            // User defined fields
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
            
            // === Common Fields (Standardized) ===
            /// Debts/liabilities that are due immediately and can be deducted.
            #[deprecated(since = "1.1.0", note = "Use `add_liability()` for granular liability tracking")]
            pub liabilities_due_now: rust_decimal::Decimal,
            /// Named liabilities for granular tracking (v1.1+).
            pub named_liabilities: Vec<$crate::types::Liability>,
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
            
            /// Sets deductible debt (deprecated, use `add_liability` instead).
            #[deprecated(since = "1.1.0", note = "Use `add_liability()` for granular liability tracking")]
            #[allow(deprecated)] // Internal usage of deprecated `liabilities_due_now`
            pub fn debt(mut self, val: impl $crate::inputs::IntoZakatDecimal) -> Self {
                match val.into_zakat_decimal() {
                    Ok(v) => self.liabilities_due_now = v,
                    Err(e) => self._input_errors.push(e),
                }
                self
            }
            
            /// Adds a named liability to the asset.
            /// 
            /// # Example
            /// ```rust,ignore
            /// let business = BusinessZakat::new()
            ///     .cash(10000)
            ///     .add_liability("Credit Card", 500)
            ///     .add_liability("Mortgage Payment", 1500);
            /// ```
            pub fn add_liability(mut self, description: impl Into<String>, amount: impl $crate::inputs::IntoZakatDecimal) -> Self {
                match amount.into_zakat_decimal() {
                    Ok(v) => {
                        self.named_liabilities.push($crate::types::Liability::new(description, v));
                    },
                    Err(e) => self._input_errors.push(e),
                }
                self
            }

            /// Adds a long-term liability (e.g., Mortgage).
            /// Only 12 months of payments will be deductible.
            pub fn add_long_term_liability(mut self, description: impl Into<String>, total_amount: impl $crate::inputs::IntoZakatDecimal, monthly_payment: impl $crate::inputs::IntoZakatDecimal) -> Self {
                let amount_res = total_amount.into_zakat_decimal();
                let monthly_res = monthly_payment.into_zakat_decimal();
                
                match (amount_res, monthly_res) {
                    (Ok(a), Ok(m)) => {
                        self.named_liabilities.push($crate::types::Liability::long_term(description, a, m));
                    },
                    (Err(e), _) => self._input_errors.push(e),
                    (_, Err(e)) => self._input_errors.push(e),
                }
                self
            }
            
            /// Returns the total *deductible* liabilities (legacy + smart calculation).
            ///
            /// # Fiqh Logic (Dayn al-Hal)
            /// - **Immediate Debt**: Fully deductible.
            /// - **Long-Term Debt**: Only the upcoming year's payments (12 months) are deductible.
            #[allow(deprecated)]
            pub fn total_liabilities(&self) -> rust_decimal::Decimal {
                // Emit deprecation warning if legacy field is used
                if self.liabilities_due_now > rust_decimal::Decimal::ZERO {
                    tracing::warn!(
                        amount = %self.liabilities_due_now,
                        "liabilities_due_now is deprecated. Use add_liability() for granular tracking."
                    );
                }

                let named_sum: rust_decimal::Decimal = self.named_liabilities.iter().map(|l| {
                    match l.kind {
                        $crate::types::LiabilityType::Immediate => l.amount,
                        $crate::types::LiabilityType::LongTerm => {
                            if let Some(monthly) = l.monthly_payment {
                                use rust_decimal_macros::dec;
                                // Deduct min(total_balance, 12 * monthly)
                                let annual_cap = monthly * dec!(12);
                                l.amount.min(annual_cap)
                            } else {
                                // Fallback if no monthly payment specified: Treat as immediate or capped?
                                // Safer to treat as full amount if data is missing, or 0? 
                                // Let's treat as full amount but warn? 
                                // For safety/correctness, if it's LongTerm but no monthly info, 
                                // we can't calculate the cap. 
                                // Defaulting to amount (conservative for the user? No, aggressive deduction).
                                // Fiqh: If unknown, assume Immediate?
                                // Let's stick to amount for now to avoid breaking changes, 
                                // but ideally strict mode would require monthly_payment.
                                l.amount
                            }
                        }
                    }
                }).sum();
                self.liabilities_due_now + named_sum
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
            /// Returns (liabilities_due_now, named_liabilities, hawl_satisfied, label, id, _input_errors, acquisition_date)
            #[allow(clippy::type_complexity)]
            fn default_common() -> (rust_decimal::Decimal, Vec<$crate::types::Liability>, bool, Option<String>, uuid::Uuid, Vec<$crate::types::ZakatError>, Option<chrono::NaiveDate>) {
                (rust_decimal::Decimal::ZERO, Vec::new(), true, None, uuid::Uuid::new_v4(), Vec::new(), None)
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
            /// Returns the optional label of the asset.
            pub fn get_label(&self) -> Option<String> { self.label.clone() }
            
            /// Returns the JSON Schema for this asset type.
            /// Useful for frontend validation and type generation.
            pub fn get_schema() -> schemars::schema::RootSchema {
                schemars::schema_for!($name)
            }
        }

        impl $crate::traits::TemporalAsset for $name {
            fn with_acquisition_date(mut self, date: chrono::NaiveDate) -> Self {
                self.acquisition_date = Some(date);
                self
            }

            fn with_hawl_satisfied(mut self, satisfied: bool) -> Self {
                self.hawl_satisfied = satisfied;
                self
            }
        }
    };
}


/// Macro for exporting Zakat assets to FFI (Python, WASM, etc.)
#[macro_export]
macro_rules! zakat_ffi_export {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        // 1. Generate the base Rust struct using zakat_asset!
        crate::zakat_asset! {
            $(#[$meta])*
            // Removed direct uniffi derive because Decimal/Uuid are not supported directly
            $vis struct $name {
                $(
                    $(#[$field_meta])*
                    $field_vis $field : $ty
                ),*
            }
        }

        // 2. Python Projection
        #[cfg(feature = "python")]
        pub mod python_ffi {
            use super::*;
            use pyo3::prelude::*;
            use pyo3::types::{PyDict};
            use rust_decimal::Decimal;
            use std::str::FromStr;
            use crate::inputs::{ToFfiString, FromFfiString};

            #[pyclass]
            #[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass)]
            #[derive(Clone)]
            pub struct $name {
                pub inner: super::$name
            }

            #[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pymethods)]
            #[pymethods]
            impl $name {
                #[new]
                #[pyo3(signature = (**kwargs))]
                pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> pyo3::PyResult<Self> {
                    let mut obj = Self { inner: super::$name::default() };
                    if let Some(k) = kwargs {
                        obj.update(k)?;
                    }
                    Ok(obj)
                }

                /// Bulk update fields using keyword arguments
                /// Example: obj.update(cash_on_hand="100", hawl_satisfied=True)
                pub fn update(&mut self, kwargs: &Bound<'_, PyDict>) -> pyo3::PyResult<()> {
                    for (key, value) in kwargs {
                        let key_str = key.extract::<String>()?;
                        let val_str = value.to_string(); 

                        match key_str.as_str() {
                            // User fields
                            $(
                                stringify!($field) => {
                                    self.inner.$field = <$ty as FromFfiString>::from_ffi_string(&val_str)
                                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid {}: {}", stringify!($field), e)))?;
                                }
                            )*
                            
                            // Common fields
                            "liabilities_due_now" | "liabilities" => {
                                #[allow(deprecated)]
                                {
                                    self.inner.liabilities_due_now = Decimal::from_str(&val_str)
                                        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid liabilities: {}", e)))?;
                                }
                            }
                            "hawl_satisfied" | "hawl" => {
                                if let Ok(b) = value.extract::<bool>() {
                                    self.inner.hawl_satisfied = b;
                                } else {
                                    let s = val_str.to_lowercase();
                                    self.inner.hawl_satisfied = s == "true" || s == "1" || s == "yes";
                                }
                            }
                            "label" => {
                                self.inner.label = Some(val_str);
                            }
                            "id" => {}
                            _ => {}
                        }
                    }
                    Ok(())
                }

                // --- Getters ---
                $(
                    #[getter]
                    pub fn $field(&self) -> String {
                        crate::inputs::ToFfiString::to_ffi_string(&self.inner.$field)
                    }
                )*

                #[getter]
                #[allow(deprecated)]
                pub fn liabilities_due_now(&self) -> String {
                    self.inner.liabilities_due_now.to_string()
                }

                #[getter]
                pub fn hawl_satisfied(&self) -> bool {
                    self.inner.hawl_satisfied
                }
                
                #[getter]
                pub fn label(&self) -> Option<String> {
                    self.inner.label.clone()
                }

                #[getter]
                pub fn _input_errors(&self) -> Vec<String> {
                    self.inner._input_errors.iter().map(|e| e.to_string()).collect::<std::vec::Vec<String>>()
                }

                fn calculate(&self, config: &$crate::python::ZakatConfig) -> pyo3::PyResult<$crate::python::ZakatDetails> {
                     use $crate::traits::CalculateZakat;
                     let details = self.inner.calculate_zakat(&config.inner)
                        .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                     Ok($crate::python::ZakatDetails { inner: details })
                }
            }
        }

        // 3. WASM Projection
        #[cfg(feature = "wasm")]
        pub mod wasm_ffi {
            use super::*;
            use wasm_bindgen::prelude::*;
            use rust_decimal::Decimal;
            use std::str::FromStr;
            use crate::inputs::{ToFfiString, FromFfiString};

            #[wasm_bindgen]
            pub struct $name {
                inner: super::$name,
            }

            #[wasm_bindgen]
            impl $name {
                #[wasm_bindgen(constructor)]
                pub fn new() -> Self {
                    Self { inner: super::$name::default() }
                }

                // --- Getters and Setters ---
                $(
                    #[wasm_bindgen(getter)]
                    pub fn $field(&self) -> String {
                        crate::inputs::ToFfiString::to_ffi_string(&self.inner.$field)
                    }
                )*
            }
            
            // We need a separate impl block for setters because we need paste! which handles the ident concatenation
            paste::paste! {
                #[wasm_bindgen]
                impl $name {
                    $(
                        #[wasm_bindgen(setter)]
                        pub fn [<set_ $field>](&mut self, val: &str) -> Result<(), JsError> {
                            self.inner.$field = <$ty as FromFfiString>::from_ffi_string(val)
                                .map_err(|e| JsError::new(&format!("Invalid {}: {}", stringify!($field), e)))?;
                            Ok(())
                        }
                    )*
                    
                    #[wasm_bindgen(setter)]
                    #[allow(deprecated)]
                    pub fn set_liabilities_due_now(&mut self, val: &str) -> Result<(), JsError> {
                         self.inner.liabilities_due_now = Decimal::from_str(val)
                            .map_err(|e| JsError::new(&format!("Invalid liabilities: {}", e)))?;
                         Ok(())
                    }
                }
            }

            #[wasm_bindgen]
            impl $name {
                 // --- Common Fields Getters (Setters handled above or separately) ---
                #[wasm_bindgen(getter)]
                #[allow(deprecated)]
                pub fn liabilities_due_now(&self) -> String {
                    self.inner.liabilities_due_now.to_string()
                }

                #[wasm_bindgen(getter)]
                pub fn hawl_satisfied(&self) -> bool {
                    self.inner.hawl_satisfied
                }

                #[wasm_bindgen(setter)]
                pub fn set_hawl_satisfied(&mut self, val: bool) {
                    self.inner.hawl_satisfied = val;
                }

                #[wasm_bindgen(getter)]
                pub fn label(&self) -> Option<String> {
                    self.inner.label.clone()
                }

                #[wasm_bindgen(setter)]
                pub fn set_label(&mut self, val: Option<String>) {
                    self.inner.label = val;
                }

                // --- Calculation ---
                pub fn calculate(&self, config_js: JsValue) -> Result<JsValue, JsValue> {
                    let config: crate::config::ZakatConfig = serde_wasm_bindgen::from_value(config_js)?;
                    use crate::traits::CalculateZakat;
                    let res = self.inner.calculate_zakat(&config)
                        .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    Ok(serde_wasm_bindgen::to_value(&res)?)
                }
            }
        }

        // 4. UniFFI Projection (Kotlin/Swift)
        #[cfg(feature = "uniffi")]
        pub mod uniffi_ffi {
            use super::*;
            use uniffi::Record;
            use crate::inputs::{ToFfiString, FromFfiString};
            
            // Mirror struct with simplified types (String for Decimal)
            #[derive(Record, Clone, Debug)]
            pub struct $name {
                // User fields
                $(
                    pub $field: String,
                )*
                // Common fields
                pub liabilities_due_now: String,
                pub hawl_satisfied: bool,
                pub label: Option<String>,
                pub id: String, // UUID as string
            }

            impl From<super::$name> for $name {
                #[allow(deprecated)]
                fn from(src: super::$name) -> Self {
                    Self {
                        $(
                           $field: ToFfiString::to_ffi_string(&src.$field),
                        )*
                        liabilities_due_now: src.liabilities_due_now.to_string(),
                        hawl_satisfied: src.hawl_satisfied,
                        label: src.label,
                        id: src.id.to_string(),
                    }
                }
            }
            
            paste::paste! {
                #[uniffi::export]
                pub fn [<calculate_ $name:snake>](asset: $name, config: &crate::config::ZakatConfig) -> Result<crate::types::FfiZakatDetails, crate::kotlin::UniFFIZakatError> {
                     #[allow(deprecated)]
                     let inner = super::$name {
                         // User fields
                         $(
                             $field: <$ty as FromFfiString>::from_ffi_string(&asset.$field)
                                 .map_err(|e| crate::kotlin::UniFFIZakatError::Generic {
                                     code: "PARSE_ERROR".to_string(),
                                     message: e.to_string(),
                                     field: Some(stringify!($field).to_string()),
                                     hint: None
                                 })?,
                         )*
                         // Common fields
                         liabilities_due_now: <rust_decimal::Decimal as FromFfiString>::from_ffi_string(&asset.liabilities_due_now)
                             .map_err(|e| crate::kotlin::UniFFIZakatError::Generic { 
                                     code: "PARSE_ERROR".to_string(),
                                     message: e.to_string(),
                                     field: Some("liabilities".into()),
                                     hint: None
                             })?,
                         named_liabilities: Vec::new(),
                         hawl_satisfied: asset.hawl_satisfied,
                         label: asset.label,
                         id: <uuid::Uuid as FromFfiString>::from_ffi_string(&asset.id)
                              .unwrap_or_else(|_| uuid::Uuid::new_v4()),
                         acquisition_date: None,
                         _input_errors: Vec::new(),
                     };
                     
                     use crate::traits::CalculateZakat;
                     let details = inner.calculate_zakat(config)
                          .map_err(|e| crate::kotlin::UniFFIZakatError::from(e))?;
                     
                     Ok(details.into())
                }
            }
        }

        // 5. Generic FFI Mirror (Plain Rust structs with String fields, for FRB etc)
        pub mod ffi_mirror {
            use super::*;
            use crate::inputs::{ToFfiString, FromFfiString};
            
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct $name {
                // User fields
                $(
                    pub $field: String,
                )*
                // Common fields
                pub liabilities_due_now: String,
                pub hawl_satisfied: bool,
                pub label: Option<String>,
                pub id: String,
            }

            #[allow(deprecated)] // Internal usage of deprecated `liabilities_due_now` for backward compat
            impl From<super::$name> for $name {
                fn from(src: super::$name) -> Self {
                    Self {
                        $(
                           $field: ToFfiString::to_ffi_string(&src.$field),
                        )*
                        liabilities_due_now: src.liabilities_due_now.to_string(),
                        hawl_satisfied: src.hawl_satisfied,
                        label: src.label,
                        id: src.id.to_string(),
                    }
                }
            }
            
            impl $name {
                #[allow(deprecated)]
                pub fn to_core(&self) -> Result<super::$name, crate::types::ZakatError> {
                     Ok(super::$name {
                         // User fields
                         $(
                             $field: <$ty as FromFfiString>::from_ffi_string(&self.$field)
                                 .map_err(|_e| crate::types::ZakatError::InvalidInput(Box::new(crate::types::InvalidInputDetails {
                                    field: stringify!($field).to_string(),
                                    value: self.$field.clone(),
                                    reason_key: "error-parse".to_string(),
                                    source_label: self.label.clone(),
                                    suggestion: Some("Ensure the value is a valid number format.".to_string()),
                                    ..Default::default()
                                 })))?,
                         )*
                         // Common fields
                         liabilities_due_now: <rust_decimal::Decimal as FromFfiString>::from_ffi_string(&self.liabilities_due_now)
                             .map_err(|_e| crate::types::ZakatError::InvalidInput(Box::new(crate::types::InvalidInputDetails{
                                field: "liabilities".to_string(),
                                value: self.liabilities_due_now.clone(),
                                reason_key: "error-parse".to_string(),
                                source_label: self.label.clone(),
                                suggestion: Some("Ensure the value is a valid number format.".to_string()),
                                ..Default::default()
                             })))?,
                         named_liabilities: Vec::new(),
                         hawl_satisfied: self.hawl_satisfied,
                         label: self.label.clone(),
                         id: <uuid::Uuid as FromFfiString>::from_ffi_string(&self.id)
                              .unwrap_or_else(|_| uuid::Uuid::new_v4()),
                         acquisition_date: None,
                         _input_errors: Vec::new(),
                     })
                }

                pub fn calculate(&self, config: &crate::config::ZakatConfig) -> Result<crate::types::FfiZakatDetails, crate::types::ZakatError> {
                     let inner = self.to_core()?;
                     
                     use crate::traits::CalculateZakat;
                     let details = inner.calculate_zakat(config)?;
                     
                     Ok(details.into())
                }
            }
        }
    };
}


// 6. Python View Implementation (Output Structs)
#[macro_export]
macro_rules! zakat_impl_py_view {
    (
        struct $core_type:path as $name:ident (name = $exposed_name:literal) {
            $(
                $field:ident : $ty:ty [$strategy:ident]
            ),* $(,)?
        }
        $(
            extra_methods {
                $($extra:tt)*
            }
        )?
    ) => {
        paste::paste! {
            #[pyo3::prelude::pyclass(name = $exposed_name)]
            #[derive(Clone, Debug)]
            pub struct $name {
                pub inner: $core_type,
            }

            #[pyo3::prelude::pymethods]
            impl $name {
                $(
                    #[getter]
                    fn [<get_ $field>](&self) -> $ty {
                        crate::zakat_impl_py_view!(@get self.inner.$field, $strategy)
                    }
                )*

                fn to_dict(&self, py: pyo3::prelude::Python) -> pyo3::prelude::PyResult<pyo3::Py<pyo3::types::PyAny>> {
                     let dict = pyo3::types::PyDict::new(py);
                     $(
                         dict.set_item(stringify!($field), self.[<get_ $field>]())?;
                     )*
                     Ok(dict.into())
                }

                fn __repr__(&self) -> String {
                    format!(
                        concat!("<", $exposed_name, " ", $(stringify!($field), "={:?} "),*, ">"),
                        $(
                            self.[<get_ $field>](),
                        )*
                    )
                }

                $(
                    $($extra)*
                )?
            }
        }
    };

    // Strategies
    (@get $expr:expr, into) => { $expr.clone().into() };
    (@get $expr:expr, to_string) => { $expr.to_string() };
    (@get $expr:expr, copy) => { $expr };
    (@get $expr:expr, clone) => { $expr.clone() };
    (@get $expr:expr, option_clone) => { $expr.clone() };
}

// 7. Python Enum Mapper
#[macro_export]
macro_rules! zakat_pymap_enum {
    (
        enum $core_type:path as $name:ident (name = $exposed_name:literal) {
            $( $variant:ident = $val:literal ),* $(,)?
        } with_impl From<$core_type_from:path> {
            $($match_pat:pat => $match_res:expr),* $(,)?
        }
    ) => {
        #[pyo3::prelude::pyclass(name = $exposed_name, eq, eq_int)]
        #[derive(Clone, PartialEq, Eq, Debug)]
        pub enum $name {
            $( $variant = $val, )*
        }

        impl From<$core_type_from> for $name {
            fn from(val: $core_type_from) -> Self {
                match val {
                    $( $match_pat => $match_res, )*
                }
            }
        }
    };
}
