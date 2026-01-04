//! Python FFI types for zakat-core
//! 
//! This module contains the basic Python FFI types that are used by the 
//! `zakat_ffi_export!` macro when generating Python bindings for asset types.

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::config::ZakatConfig;

#[cfg(feature = "python")]
use crate::types::{WealthType, ZakatDetails};

// -----------------------------------------------------------------------------
// PyZakatConfig - Python wrapper for ZakatConfig
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "ZakatConfig")]
#[derive(Clone)]
pub struct PyZakatConfig {
    pub inner: ZakatConfig,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyZakatConfig {
    #[new]
    #[pyo3(signature = (gold_price, silver_price, rice_price_kg=None, rice_price_liter=None))]
    pub fn new(
        gold_price: &Bound<'_, PyAny>,
        silver_price: &Bound<'_, PyAny>,
        rice_price_kg: Option<&Bound<'_, PyAny>>,
        rice_price_liter: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        let gold = extract_decimal(gold_price, "gold_price")?;
        let silver = extract_decimal(silver_price, "silver_price")?;

        let mut config = ZakatConfig::hanafi(gold, silver);

        if let Some(price) = rice_price_kg {
            let p = extract_decimal(price, "rice_price_kg")?;
            config = config.with_rice_price_per_kg(p);
        }
        
        if let Some(price) = rice_price_liter {
            let p = extract_decimal(price, "rice_price_liter")?;
            config = config.with_rice_price_per_liter(p);
        }

        Ok(PyZakatConfig { inner: config })
    }



    #[getter]
    fn gold_price_per_gram(&self) -> String {
        self.inner.gold_price_per_gram.to_string()
    }

    #[getter]
    fn silver_price_per_gram(&self) -> String {
        self.inner.silver_price_per_gram.to_string()
    }

    #[staticmethod]
    fn is_valid_input(val: &str) -> bool {
        crate::inputs::validate_numeric_format(val)
    }
}

/// Helper to convert a Python object to ZakatDecimal (rust_decimal::Decimal).
/// Supports: str, int, float, decimal.Decimal
fn extract_decimal(obj: &Bound<'_, PyAny>, field_name: &str) -> PyResult<rust_decimal::Decimal> {
    use crate::inputs::IntoZakatDecimal;
    
    // 1. Try string directly
    if let Ok(s) = obj.extract::<String>() {
        return s.clone().into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid {} string '{}': {}", field_name, s, e)));
    }

    // 2. Try float (and convert via string to preserve precision if possible or standard float conversion)
    // ZakatDecimal usually prefers string for precision, but f64 is acceptable if user provides it.
    if let Ok(_f) = obj.extract::<f64>() {
        // Use string intermediate to avoid some binary float edge cases, or use form_f64
        // Logic: rust_decimal::Decimal::from_f64_retain is usually good.
        // But for "currency", string is safer.
        // Let's coerce to string on Python side? No, `str(obj)` is safer.
        let s = obj.str()?.to_string();
        return s.clone().into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid {} float '{}': {}", field_name, s, e)));
    }
    
    // 3. Try integer
    if let Ok(i) = obj.extract::<i64>() {
        return Ok(rust_decimal::Decimal::from(i));
    }
    
    // 4. Try Python Decimal (if strictly typed) - calling __str__ is the safest generic way
    // (covered by step 2 fallback essentially if we just call str())
    let s = obj.str()?.to_string();
    s.clone().into_zakat_decimal()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid {} value '{}': {}", field_name, s, e)))
}

// -----------------------------------------------------------------------------
// PyWealthType - Python wrapper for WealthType enum
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "WealthType")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyWealthType {
    Gold = 0,
    Silver = 1,
    Business = 2,
    Agriculture = 3,
    Livestock = 4,
    Mining = 5,
    Income = 6,
    Investment = 7,
    Fitrah = 8,
}

#[cfg(feature = "python")]
impl From<WealthType> for PyWealthType {
    fn from(wt: WealthType) -> Self {
        match wt {
            WealthType::Gold => PyWealthType::Gold,
            WealthType::Silver => PyWealthType::Silver,
            WealthType::Business => PyWealthType::Business,
            WealthType::Agriculture => PyWealthType::Agriculture,
            WealthType::Livestock => PyWealthType::Livestock,
            WealthType::Mining => PyWealthType::Mining,
            WealthType::Income => PyWealthType::Income,
            WealthType::Investment => PyWealthType::Investment,
            WealthType::Fitrah => PyWealthType::Fitrah,
            WealthType::Rikaz | WealthType::Other(_) => PyWealthType::Business,
        }
    }
}

// -----------------------------------------------------------------------------
// PyZakatDetails - Python wrapper for ZakatDetails
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "ZakatDetails")]
#[derive(Clone)]
pub struct PyZakatDetails {
    pub inner: ZakatDetails,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyZakatDetails {
    #[getter]
    fn wealth_type(&self) -> PyWealthType {
        PyWealthType::from(self.inner.wealth_type.clone())
    }

    #[getter]
    fn net_assets(&self) -> String {
        self.inner.net_assets.to_string()
    }

    #[getter]
    fn zakat_due(&self) -> String {
        self.inner.zakat_due.to_string()
    }

    #[getter]
    fn total_assets(&self) -> String {
        self.inner.total_assets.to_string()
    }

    #[getter]
    fn is_payable(&self) -> bool {
        self.inner.is_payable
    }

    #[getter]
    fn nisab_threshold(&self) -> String {
        self.inner.nisab_threshold.to_string()
    }

    #[getter]
    fn status_reason(&self) -> Option<String> {
        self.inner.status_reason.clone()
    }
}

// -----------------------------------------------------------------------------
// PyPreciousMetals - Python wrapper for PreciousMetals
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::precious_metals::python_ffi::PreciousMetals as PyPreciousMetals;

// -----------------------------------------------------------------------------
// PyBusinessZakat - Python wrapper for BusinessZakat
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::business::python_ffi::BusinessZakat as PyBusinessZakat;

// -----------------------------------------------------------------------------
// PyInvestmentAssets - Python wrapper for InvestmentAssets
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::investments::python_ffi::InvestmentAssets as PyInvestmentAssets;

// -----------------------------------------------------------------------------
// PyIncomeZakatCalculator - Python wrapper for IncomeZakatCalculator
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::income::python_ffi::IncomeZakatCalculator as PyIncomeZakatCalculator;

// -----------------------------------------------------------------------------
// PyMiningAssets - Python wrapper for MiningAssets
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::mining::python_ffi::MiningAssets as PyMiningAssets;

// -----------------------------------------------------------------------------
// Main Python module definition
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pymodule]
fn zakatrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyZakatConfig>()?;
    m.add_class::<PyWealthType>()?;
    m.add_class::<PyZakatDetails>()?;
    m.add_class::<PyPreciousMetals>()?;
    m.add_class::<PyBusinessZakat>()?;
    m.add_class::<PyIncomeZakatCalculator>()?;
    m.add_class::<PyInvestmentAssets>()?;
    m.add_class::<PyMiningAssets>()?;
    Ok(())
}
