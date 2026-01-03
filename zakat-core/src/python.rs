//! Python FFI types for zakat-core
//! 
//! This module contains the basic Python FFI types that are used by the 
//! `zakat_ffi_export!` macro when generating Python bindings for asset types.

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use pyo3::types::PyDict;

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
        gold_price: &str,
        silver_price: &str,
        rice_price_kg: Option<&str>,
        rice_price_liter: Option<&str>,
    ) -> PyResult<Self> {
        use crate::inputs::IntoZakatDecimal;
        let gold = gold_price.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid gold price '{}': {}", gold_price, e)))?;
        let silver = silver_price.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid silver price '{}': {}", silver_price, e)))?;

        let mut config = ZakatConfig::hanafi(gold, silver);

        if let Some(price) = rice_price_kg {
            let p = price.into_zakat_decimal()
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid rice price (kg) '{}': {}", price, e)))?;
            config = config.with_rice_price_per_kg(p);
        }
        
        if let Some(price) = rice_price_liter {
            let p = price.into_zakat_decimal()
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid rice price (liter) '{}': {}", price, e)))?;
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
