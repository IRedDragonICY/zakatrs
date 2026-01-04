//! Python FFI types for zakat-core
//! 
//! This module contains the basic Python FFI types that are used by the 
//! `zakat_ffi_export!` macro when generating Python bindings for asset types.

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::config::ZakatConfig as CoreZakatConfig;

#[cfg(feature = "python")]
use crate::types::{WealthType as CoreWealthType, ZakatDetails as CoreZakatDetails};

#[cfg(feature = "stub-gen")]
use pyo3_stub_gen::derive::*; // For #[gen_stub_pyclass]

// -----------------------------------------------------------------------------
// ZakatConfig - Python wrapper for ZakatConfig
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "ZakatConfig")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[derive(Clone)]
pub struct ZakatConfig {
    pub inner: CoreZakatConfig,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl ZakatConfig {
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

        let mut config = CoreZakatConfig::hanafi(gold, silver);

        if let Some(price) = rice_price_kg {
            let p = extract_decimal(price, "rice_price_kg")?;
            config = config.with_rice_price_per_kg(p);
        }
        
        if let Some(price) = rice_price_liter {
            let p = extract_decimal(price, "rice_price_liter")?;
            config = config.with_rice_price_per_liter(p);
        }

        Ok(ZakatConfig { inner: config })
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

    #[staticmethod]
    pub fn hanafi(gold_price: &Bound<'_, PyAny>, silver_price: &Bound<'_, PyAny>) -> PyResult<Self> {
        let gold = extract_decimal(gold_price, "gold_price")?;
        let silver = extract_decimal(silver_price, "silver_price")?;
        Ok(ZakatConfig { inner: CoreZakatConfig::hanafi(gold, silver) })
    }

    #[staticmethod]
    pub fn shafi(gold_price: &Bound<'_, PyAny>) -> PyResult<Self> {
        let gold = extract_decimal(gold_price, "gold_price")?;
        Ok(ZakatConfig { inner: CoreZakatConfig::shafi(gold) })
    }

    #[staticmethod]
    pub fn maliki(gold_price: &Bound<'_, PyAny>) -> PyResult<Self> {
        let gold = extract_decimal(gold_price, "gold_price")?;
        Ok(ZakatConfig { inner: CoreZakatConfig::maliki(gold) })
    }

    #[staticmethod]
    pub fn hanbali(gold_price: &Bound<'_, PyAny>) -> PyResult<Self> {
        let gold = extract_decimal(gold_price, "gold_price")?;
        Ok(ZakatConfig { inner: CoreZakatConfig::hanbali(gold) })
    }

    #[staticmethod]
    pub fn for_region(iso_code: String) -> Self {
        ZakatConfig { inner: CoreZakatConfig::for_region(&iso_code) }
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
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WealthType {
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
impl From<CoreWealthType> for WealthType {
    fn from(wt: CoreWealthType) -> Self {
        match wt {
            CoreWealthType::Gold => WealthType::Gold,
            CoreWealthType::Silver => WealthType::Silver,
            CoreWealthType::Business => WealthType::Business,
            CoreWealthType::Agriculture => WealthType::Agriculture,
            CoreWealthType::Livestock => WealthType::Livestock,
            CoreWealthType::Mining => WealthType::Mining,
            CoreWealthType::Income => WealthType::Income,
            CoreWealthType::Investment => WealthType::Investment,
            CoreWealthType::Fitrah => WealthType::Fitrah,
            CoreWealthType::Rikaz | CoreWealthType::Other(_) => WealthType::Business,
        }
    }
}

// -----------------------------------------------------------------------------
// PyZakatRecommendation
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "ZakatRecommendation")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZakatRecommendation {
    Obligatory = 0,
    Recommended = 1,
    None = 2,
}

#[cfg(feature = "python")]
impl From<crate::types::ZakatRecommendation> for ZakatRecommendation {
    fn from(rec: crate::types::ZakatRecommendation) -> Self {
        match rec {
            crate::types::ZakatRecommendation::Obligatory => ZakatRecommendation::Obligatory,
            crate::types::ZakatRecommendation::Recommended => ZakatRecommendation::Recommended,
            crate::types::ZakatRecommendation::None => ZakatRecommendation::None,
        }
    }
}

// -----------------------------------------------------------------------------
// PyLiabilityType
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "LiabilityType")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LiabilityType {
    Immediate = 0,
    LongTerm = 1,
}

// -----------------------------------------------------------------------------
// PyWarningCode
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "WarningCode")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass_enum)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WarningCode {
    NegativeAssetsClamped = 0,
    GrossMethodExpensesIgnored = 1,
    LivestockBelowNisab = 2,
    MetalBelowNisab = 3,
    PriceDataStale = 4,
    HawlNotMet = 5,
    PartialCalculation = 6,
    CurrencyConversionApplied = 7,
    Other = 8,
}

// -----------------------------------------------------------------------------
// ZakatDetails - Python wrapper for ZakatDetails
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pyclass(name = "ZakatDetails")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pyclass)]
#[derive(Clone)]
pub struct ZakatDetails {
    pub inner: CoreZakatDetails,
}

#[cfg(feature = "python")]
#[cfg_attr(feature = "stub-gen", pyo3_stub_gen::derive::gen_stub_pymethods)]
#[pymethods]
impl ZakatDetails {
    #[getter]
    fn wealth_type(&self) -> WealthType {
        WealthType::from(self.inner.wealth_type.clone())
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

    #[getter]
    fn recommendation(&self) -> String {
        format!("{:?}", self.inner.recommendation)
    }

    #[getter]
    fn structured_warnings(&self, py: Python<'_>) -> PyResult<PyObject> {
        use pyo3::types::{PyDict, PyList};
        let list = PyList::empty(py);
        for w in &self.inner.structured_warnings {
            let dict = PyDict::new(py);
            dict.set_item("code", format!("{:?}", w.code))?;
            dict.set_item("message", &w.message)?;
            if let Some(details) = &w.details {
                let d = PyDict::new(py);
                for (k, v) in details {
                    d.set_item(k, v)?;
                }
                dict.set_item("details", d)?;
            }
            list.append(dict)?;
        }
        Ok(list.into())
    }
}

// -----------------------------------------------------------------------------
// PyPreciousMetals - Python wrapper for PreciousMetals
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::precious_metals::python_ffi::PreciousMetals;

// -----------------------------------------------------------------------------
// PyBusinessZakat - Python wrapper for BusinessZakat
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::business::python_ffi::BusinessZakat;

// -----------------------------------------------------------------------------
// PyInvestmentAssets - Python wrapper for InvestmentAssets
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::investments::python_ffi::InvestmentAssets;

// -----------------------------------------------------------------------------
// PyIncomeZakatCalculator - Python wrapper for IncomeZakatCalculator
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::income::python_ffi::IncomeZakatCalculator;

// -----------------------------------------------------------------------------
// PyMiningAssets - Python wrapper for MiningAssets
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
pub use crate::maal::mining::python_ffi::MiningAssets;

// -----------------------------------------------------------------------------
// Main Python module definition
// -----------------------------------------------------------------------------
#[cfg(feature = "python")]
#[pymodule]
fn zakatrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ZakatConfig>()?;
    m.add_class::<WealthType>()?;
    m.add_class::<ZakatDetails>()?;
    m.add_class::<PreciousMetals>()?;
    m.add_class::<BusinessZakat>()?;
    m.add_class::<IncomeZakatCalculator>()?;
    m.add_class::<InvestmentAssets>()?;
    m.add_class::<MiningAssets>()?;
    m.add_class::<ZakatRecommendation>()?;
    m.add_class::<LiabilityType>()?;
    m.add_class::<WarningCode>()?;
    Ok(())
}

#[cfg(all(feature = "python", feature = "stub-gen"))]
pub fn stub_info() -> pyo3_stub_gen::StubInfo {
    use pyo3_stub_gen::define_stub_info_gatherer;
    define_stub_info_gatherer!(internal_stub_info);
    internal_stub_info().expect("Failed to generate stub info")
}
