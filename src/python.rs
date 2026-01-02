use pyo3::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::config::ZakatConfig;
use crate::types::WealthType;
use pyo3::types::{PyAny, PyDict};

/// Python wrapper for ZakatConfig
#[pyclass(name = "ZakatConfig")]
#[derive(Clone, Debug)]
pub struct PyZakatConfig {
    pub inner: ZakatConfig,
}

#[pymethods]
impl PyZakatConfig {
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
    fn get_gold_price(&self) -> String {
        self.inner.gold_price_per_gram.to_string()
    }

    #[getter]
    fn get_silver_price(&self) -> String {
        self.inner.silver_price_per_gram.to_string()
    }
    
    fn __repr__(&self) -> String {
        format!(
            "<ZakatConfig gold={} silver={}>",
            self.inner.gold_price_per_gram, self.inner.silver_price_per_gram
        )
    }
}

/// Wealth Type Enum Wrapper
#[pyclass(name = "WealthType", eq)]
#[derive(Clone, PartialEq, Eq, Debug)]
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
            WealthType::Rikaz | WealthType::Other(_) => PyWealthType::Business, // Fallback
        }
    }
}

/// Python wrapper for ZakatDetails
#[pyclass(name = "ZakatDetails")]
#[derive(Clone, Debug)]
pub struct PyZakatDetails {
    pub inner: crate::types::ZakatDetails,
}

#[pymethods]
impl PyZakatDetails {
    #[getter]
    fn get_wealth_type(&self) -> PyWealthType {
        self.inner.wealth_type.clone().into()
    }
    
    #[getter]
    fn get_net_assets(&self) -> String {
        self.inner.net_assets.to_string()
    }
    
    #[getter]
    fn get_zakat_due(&self) -> String {
        self.inner.zakat_due.to_string()
    }
    
    #[getter]
    fn get_total_assets(&self) -> String {
        self.inner.total_assets.to_string()
    }

    #[getter]
    fn get_is_payable(&self) -> bool {
        self.inner.is_payable
    }
    
    #[getter]
    fn get_nisab_threshold(&self) -> String {
        self.inner.nisab_threshold.to_string()
    }

    #[getter]
    fn get_status_reason(&self) -> Option<String> {
        self.inner.status_reason.clone()
    }
    
    /// Returns the data as a Python dictionary
    fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item("wealth_type", self.get_wealth_type())?;
        dict.set_item("net_assets", self.inner.net_assets.to_string())?;
        dict.set_item("zakat_due", self.inner.zakat_due.to_string())?;
        dict.set_item("total_assets", self.inner.total_assets.to_string())?;
        dict.set_item("is_payable", self.inner.is_payable)?;
        dict.set_item("nisab_threshold", self.inner.nisab_threshold.to_string())?;
        dict.set_item("status_reason", self.inner.status_reason.clone())?;
        Ok(dict.into())
    }
    
    fn __repr__(&self) -> String {
        format!(
            "<ZakatDetails type={:?} is_payable={} due={}>",
            self.inner.wealth_type, self.inner.is_payable, self.inner.zakat_due
        )
    }
}

// ================= ASSET WRAPPERS =================

/// Wrapper for PreciousMetals
#[pyclass(name = "PreciousMetals")]
#[derive(Clone)]
pub struct PyPreciousMetals {
    inner: crate::maal::precious_metals::PreciousMetals,
}

#[pymethods]
impl PyPreciousMetals {
    #[staticmethod]
    fn is_valid_input(val: &str) -> bool {
        crate::inputs::validate_numeric_format(val)
    }

    #[new]
    #[pyo3(signature = (weight="0", metal_type="gold", purity=24))]
    fn new(weight: &str, metal_type: &str, purity: u32) -> PyResult<Self> {
        use crate::inputs::IntoZakatDecimal;
        let w = weight.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid weight '{}': {}", weight, e)))?;
        
        let metal = match metal_type.to_lowercase().as_str() {
            "silver" => crate::maal::precious_metals::PreciousMetals::silver(w),
            _ => crate::maal::precious_metals::PreciousMetals::gold(w).purity(purity),
        };
        Ok(PyPreciousMetals {
            inner: metal
        })
    }
    
    fn calculate(&self, config: &PyZakatConfig) -> PyResult<PyZakatDetails> {
        use crate::traits::CalculateZakat;
        let details = self.inner.calculate_zakat(&config.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyZakatDetails { inner: details })
    }
}

/// Wrapper for BusinessZakat (Trade Goods)
#[pyclass(name = "BusinessZakat")]
#[derive(Clone)]
pub struct PyBusinessZakat {
    inner: crate::maal::business::BusinessZakat,
}

#[pymethods]
impl PyBusinessZakat {
    #[staticmethod]
    fn is_valid_input(val: &str) -> bool {
        crate::inputs::validate_numeric_format(val)
    }

    #[new]
    #[pyo3(signature = (cash="0", merchandise="0", receivables="0", liabilities="0"))]
    fn new(cash: &str, merchandise: &str, receivables: &str, liabilities: &str) -> PyResult<Self> {
        use crate::inputs::IntoZakatDecimal;
        let c = cash.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid cash '{}': {}", cash, e)))?;
        let m = merchandise.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid merchandise '{}': {}", merchandise, e)))?;
        let r = receivables.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid receivables '{}': {}", receivables, e)))?;
        let l = liabilities.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid liabilities '{}': {}", liabilities, e)))?;
        
        Ok(PyBusinessZakat {
            inner: crate::maal::business::BusinessZakat::new()
                .cash(c)
                .inventory(m)
                .receivables(r)
                .liabilities(l)
        })
    }

    fn calculate(&self, config: &PyZakatConfig) -> PyResult<PyZakatDetails> {
        use crate::traits::CalculateZakat;
        let details = self.inner.calculate_zakat(&config.inner)
             .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyZakatDetails { inner: details })
    }
}

/// Wrapper for InvestmentAssets (Stocks, Crypto, etc.)
#[pyclass(name = "InvestmentAssets")]
#[derive(Clone)]
pub struct PyInvestmentAssets {
    inner: crate::maal::investments::InvestmentAssets,
}

#[pymethods]
impl PyInvestmentAssets {
    #[staticmethod]
    fn is_valid_input(val: &str) -> bool {
        crate::inputs::validate_numeric_format(val)
    }

    #[new]
    #[pyo3(signature = (value="0", investment_type="stock", hawl_satisfied=true))]
    fn new(value: &str, investment_type: &str, hawl_satisfied: bool) -> PyResult<Self> {
        use crate::inputs::IntoZakatDecimal;
        let v = value.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid value '{}': {}", value, e)))?;
        
        let kind = match investment_type.to_lowercase().as_str() {
            "crypto" => crate::maal::investments::InvestmentType::Crypto,
            "mutualfund" | "mutual_fund" => crate::maal::investments::InvestmentType::MutualFund,
            _ => crate::maal::investments::InvestmentType::Stock,
        };
        
        Ok(PyInvestmentAssets {
            inner: crate::maal::investments::InvestmentAssets::new()
                .value(v)
                .kind(kind)
                .hawl(hawl_satisfied)
        })
    }
    
    fn calculate(&self, config: &PyZakatConfig) -> PyResult<PyZakatDetails> {
        use crate::traits::CalculateZakat;
         let details = self.inner.calculate_zakat(&config.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyZakatDetails { inner: details })
    }
}

/// Wrapper for IncomeZakatCalculator
#[pyclass(name = "IncomeZakatCalculator")]
#[derive(Clone)]
pub struct PyIncomeZakatCalculator {
    inner: crate::maal::income::IncomeZakatCalculator,
}

#[pymethods]
impl PyIncomeZakatCalculator {
    #[staticmethod]
    fn is_valid_input(val: &str) -> bool {
        crate::inputs::validate_numeric_format(val)
    }

    #[new]
    #[pyo3(signature = (income, expenses="0", method="gross"))]
    fn new(income: &str, expenses: &str, method: &str) -> PyResult<Self> {
        use crate::inputs::IntoZakatDecimal;
        let i = income.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid income '{}': {}", income, e)))?;
        let e = expenses.into_zakat_decimal()
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid expenses '{}': {}", expenses, e)))?;
        
        let m = match method.to_lowercase().as_str() {
            "net" => crate::maal::income::IncomeCalculationMethod::Net,
            _ => crate::maal::income::IncomeCalculationMethod::Gross,
        };
        
        Ok(PyIncomeZakatCalculator {
             inner: crate::maal::income::IncomeZakatCalculator::new()
                .income(i)
                .expenses(e)
                .method(m)
                .hawl(true) // Default to true as per Rust factory
        })
    }
    
    fn calculate(&self, config: &PyZakatConfig) -> PyResult<PyZakatDetails> {
        use crate::traits::CalculateZakat;
         let details = self.inner.calculate_zakat(&config.inner)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        Ok(PyZakatDetails { inner: details })
    }
}

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
    m.add_class::<PyZakatPortfolio>()?;
    m.add_class::<PyPortfolioResult>()?;
    Ok(())
}
