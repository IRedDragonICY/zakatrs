use wasm_bindgen::prelude::*;
use crate::prelude::*;
use crate::config::ZakatConfig;
use crate::portfolio::ZakatPortfolio;
use crate::assets::PortfolioItem;
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};

/// Initialize hooks for better debugging in WASM
#[wasm_bindgen]
pub fn init_hooks() {
    console_error_panic_hook::set_once();
}

/// Calculate Zakat for a portfolio
/// 
/// Adapts the Rust `ZakatPortfolio::calculate_total` to JS.
/// 
/// # Arguments
/// - `config_json`: `ZakatConfig` object
/// - `assets_json`: Array of `PortfolioItem` objects
#[wasm_bindgen]
#[derive(Serialize)]
struct WasmZakatError {
    code: String,
    message: String,
    field: Option<String>,
    hint: Option<String>,
}

impl From<crate::types::ZakatError> for WasmZakatError {
    fn from(err: crate::types::ZakatError) -> Self {
        // Use default translator (English) for now, or could potentially expose locale setting in future
        let message = err.report_default(); 
        
        match err {
            crate::types::ZakatError::CalculationError(details) => WasmZakatError {
                code: "CALCULATION_ERROR".to_string(),
                message, 
                field: details.source_label,
                hint: None,
            },
            crate::types::ZakatError::InvalidInput(details) => WasmZakatError {
                code: "INVALID_INPUT".to_string(),
                message,
                field: Some(details.field),
                hint: details.source_label,
            },
            crate::types::ZakatError::ConfigurationError(details) => WasmZakatError {
                code: "CONFIG_ERROR".to_string(),
                message,
                field: details.source_label,
                hint: None,
            },
            crate::types::ZakatError::MissingConfig { field, source_label, .. } => WasmZakatError {
                code: "MISSING_CONFIG".to_string(),
                message,
                field: Some(field),
                hint: source_label,
            },
            crate::types::ZakatError::Overflow { source_label, .. } => WasmZakatError {
                code: "OVERFLOW".to_string(),
                message,
                field: source_label,
                hint: None,
            },
            crate::types::ZakatError::MultipleErrors(errs) => WasmZakatError {
                 code: "MULTIPLE_ERRORS".to_string(),
                 message: format!("{} errors occurred: {}", errs.len(), message),
                 field: None,
                 hint: None,
            },
            crate::types::ZakatError::NetworkError(_) => WasmZakatError {
                code: "NETWORK_ERROR".to_string(),
                message,
                field: None,
                hint: None,
            },
        }
    }
}

/// Calculate Zakat for a portfolio
/// 
/// Adapts the Rust `ZakatPortfolio::calculate_total` to JS.
/// 
/// # Arguments
/// - `config_json`: `ZakatConfig` object
/// - `assets_json`: Array of `PortfolioItem` objects
#[wasm_bindgen]
pub fn calculate_portfolio_wasm(config_json: JsValue, assets_json: JsValue) -> Result<JsValue, JsValue> {
    let config: ZakatConfig = from_value(config_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Config JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: JSON Error serialization failed"))
        })?;
        
    let assets: Vec<PortfolioItem> = from_value(assets_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Assets JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: JSON Error serialization failed"))
        })?;

    let mut portfolio = ZakatPortfolio::new();
    for asset in assets {
        portfolio = portfolio.add(asset);
    }
    
    let result = portfolio.calculate_total(&config);
    
    to_value(&result)
        .map_err(|e| {
             let err = WasmZakatError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: format!("Failed to serialize result: {}", e),
                field: None,
                hint: None,
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: Result serialization failed"))
        })
}

/// Helper: Calculate Zakat for a single asset just like the portfolio but simpler
#[wasm_bindgen]
pub fn calculate_single_asset(config_json: JsValue, asset_json: JsValue) -> Result<JsValue, JsValue> {
    let config: ZakatConfig = from_value(config_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Config JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: Config serialization failed"))
        })?;
    
    let asset: PortfolioItem = from_value(asset_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Asset JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: Asset serialization failed"))
        })?;

    let details = asset.calculate_zakat(&config)
        .map_err(|e| {
            let wasm_err: WasmZakatError = e.into();
            serde_wasm_bindgen::to_value(&wasm_err).unwrap_or_else(|_| JsValue::from_str("Critical: Zakat Error serialization failed"))
        })?;
        
    to_value(&details)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: format!("Failed to serialize result: {}", e),
                field: None,
                hint: None,
            };
            serde_wasm_bindgen::to_value(&err).unwrap_or_else(|_| JsValue::from_str("Critical: Final Result serialization failed"))
        })
}

/// Helper: Test if WASM is alive
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Zakat WASM is ready.", name)
}
