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
        let context = err.context();
        let code = context["code"].as_str().unwrap_or("UNKNOWN_ERROR").to_string();
        let message = context["message"].as_str().unwrap_or("An unknown error occurred").to_string();
        let field = context.get("field").and_then(|v| v.as_str()).map(|s| s.to_string());
        let hint = context.get("hint").and_then(|v| v.as_str()).map(|s| s.to_string());

        WasmZakatError {
            code,
            message,
            field,
            hint,
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
            serde_wasm_bindgen::to_value(&err).unwrap()
        })?;
        
    let assets: Vec<PortfolioItem> = from_value(assets_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Assets JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap()
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
            serde_wasm_bindgen::to_value(&err).unwrap()
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
            serde_wasm_bindgen::to_value(&err).unwrap()
        })?;
    
    let asset: PortfolioItem = from_value(asset_json)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "JSON_ERROR".to_string(),
                message: format!("Invalid Asset JSON: {}", e),
                field: None,
                hint: Some("Check JSON format".to_string()),
            };
            serde_wasm_bindgen::to_value(&err).unwrap()
        })?;

    let details = asset.calculate_zakat(&config)
        .map_err(|e| {
            let wasm_err: WasmZakatError = e.into();
            serde_wasm_bindgen::to_value(&wasm_err).unwrap()
        })?;
        
    to_value(&details)
        .map_err(|e| {
            let err = WasmZakatError {
                code: "SERIALIZATION_ERROR".to_string(),
                message: format!("Failed to serialize result: {}", e),
                field: None,
                hint: None,
            };
            serde_wasm_bindgen::to_value(&err).unwrap()
        })
}

/// Helper: Test if WASM is alive
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Zakat WASM is ready.", name)
}
