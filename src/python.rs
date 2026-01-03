use pyo3::prelude::*;

use crate::config::ZakatConfig;
use crate::types::WealthType;
use pyo3::types::{PyAny, PyDict};

// -----------------------------------------------------------------------------
// Auto-generated PyZakatConfig
// -----------------------------------------------------------------------------
crate::zakat_impl_py_view! {
    struct crate::config::ZakatConfig as PyZakatConfig (name = "ZakatConfig") {
        gold_price_per_gram: String [to_string],
        silver_price_per_gram: String [to_string],
    }
    extra_methods {
        #[staticmethod]
        fn is_valid_input(val: &str) -> bool {
            crate::inputs::validate_numeric_format(val)
        }

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

            let mut config = crate::config::ZakatConfig::hanafi(gold, silver);

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
    }
}

// -----------------------------------------------------------------------------
// Auto-generated WealthType Enum
// -----------------------------------------------------------------------------
crate::zakat_pymap_enum! {
    enum crate::types::WealthType as PyWealthType (name = "WealthType") {
        Gold = 0,
        Silver = 1,
        Business = 2,
        Agriculture = 3,
        Livestock = 4,
        Mining = 5,
        Income = 6,
        Investment = 7,
        Fitrah = 8,
    } with_impl From<crate::types::WealthType> {
        crate::types::WealthType::Gold => PyWealthType::Gold,
        crate::types::WealthType::Silver => PyWealthType::Silver,
        crate::types::WealthType::Business => PyWealthType::Business,
        crate::types::WealthType::Agriculture => PyWealthType::Agriculture,
        crate::types::WealthType::Livestock => PyWealthType::Livestock,
        crate::types::WealthType::Mining => PyWealthType::Mining,
        crate::types::WealthType::Income => PyWealthType::Income,
        crate::types::WealthType::Investment => PyWealthType::Investment,
        crate::types::WealthType::Fitrah => PyWealthType::Fitrah,
        // Fallback or explicit mapping for complex variants
        crate::types::WealthType::Rikaz | crate::types::WealthType::Other(_) => PyWealthType::Business,
    }
}

// -----------------------------------------------------------------------------
// Auto-generated PyZakatDetails using zakat_impl_py_view!
// -----------------------------------------------------------------------------
crate::zakat_impl_py_view! {
    struct crate::types::ZakatDetails as PyZakatDetails (name = "ZakatDetails") {
        wealth_type: PyWealthType [into],
        net_assets: String [to_string],
        zakat_due: String [to_string],
        total_assets: String [to_string],
        is_payable: bool [copy],
        nisab_threshold: String [to_string],
        status_reason: Option<String> [option_clone],
    }
}

// ================= ASSET WRAPPERS =================

/// Wrapper for PreciousMetals
/// Now generated automatically by zakat_ffi_export! in src/maal/precious_metals.rs
pub use crate::maal::precious_metals::python_ffi::PreciousMetals as PyPreciousMetals;

/// Wrapper for BusinessZakat (Trade Goods)
/// Note: Now generated automatically by zakat_ffi_export! in src/maal/business.rs
pub use crate::maal::business::python_ffi::BusinessZakat as PyBusinessZakat;

/// Wrapper for InvestmentAssets (Stocks, Crypto, etc.)
/// Now generated automatically by zakat_ffi_export! in src/maal/investments.rs
pub use crate::maal::investments::python_ffi::InvestmentAssets as PyInvestmentAssets;

/// Wrapper for IncomeZakatCalculator
/// Now generated automatically by zakat_ffi_export! in src/maal/income.rs
pub use crate::maal::income::python_ffi::IncomeZakatCalculator as PyIncomeZakatCalculator;

/// Wrapper for MiningAssets
pub use crate::maal::mining::python_ffi::MiningAssets as PyMiningAssets;

// ================= PORTFOLIO =================

#[pyclass(name = "ZakatPortfolio")]
#[derive(Clone)]
pub struct PyZakatPortfolio {
    inner: crate::portfolio::ZakatPortfolio,
}

#[pymethods]
impl PyZakatPortfolio {
    #[new]
    fn new() -> Self {
        PyZakatPortfolio { inner: crate::portfolio::ZakatPortfolio::new() }
    }

    fn add(&mut self, item: &Bound<'_, PyAny>) -> PyResult<()> {
        if let Ok(asset) = item.extract::<PyBusinessZakat>() {
            self.inner.push(asset.inner.clone());
        } else if let Ok(asset) = item.extract::<PyPreciousMetals>() {
             self.inner.push(asset.inner.clone());
        } else if let Ok(asset) = item.extract::<PyInvestmentAssets>() {
             self.inner.push(asset.inner.clone());
        } else if let Ok(asset) = item.extract::<PyIncomeZakatCalculator>() {
             self.inner.push(asset.inner.clone());
        } else if let Ok(asset) = item.extract::<PyMiningAssets>() {
             self.inner.push(asset.inner.clone());
        } else {
             return Err(pyo3::exceptions::PyTypeError::new_err("Unsupported asset type"));
        }
        Ok(())
    }

    fn calculate(&self, config: &PyZakatConfig) -> PyResult<PyPortfolioResult> {
        let res = self.inner.calculate_total(&config.inner);
        Ok(PyPortfolioResult { inner: res })
    }
}

#[pyclass(name = "PortfolioResult")]
#[derive(Clone)]
pub struct PyPortfolioResult {
    inner: crate::portfolio::PortfolioResult,
}

#[pymethods]
impl PyPortfolioResult {
    #[getter]
    fn get_total_zakat_due(&self) -> String {
        self.inner.total_zakat_due.to_string()
    }
    
    #[getter]
    fn get_total_assets(&self) -> String {
        self.inner.total_assets.to_string()
    }
    
    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("total_zakat_due", self.inner.total_zakat_due.to_string())?;
        dict.set_item("total_assets", self.inner.total_assets.to_string())?;
        Ok(dict.into())
    }
}


/// Main module entry point (UPDATED)
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
    m.add_class::<PyZakatPortfolio>()?;
    m.add_class::<PyPortfolioResult>()?;
    Ok(())
}
