use wasm_bindgen::prelude::*;
use crate::prelude::*;
use crate::config::ZakatConfig;
use crate::portfolio::ZakatPortfolio;
use crate::assets::PortfolioItem;
use serde_wasm_bindgen::{from_value, to_value};

// Re-export the centralized FFI error type
// No more manual maintenance needed - if ZakatError changes, FfiZakatError handles it!
use crate::types::FfiZakatError;

/// Initialize hooks for better debugging in WASM
#[wasm_bindgen]
pub fn init_hooks() {
    console_error_panic_hook::set_once();
}

/// Helper to create a JSON parsing error
fn json_error(message: String, hint: Option<String>) -> JsValue {
    let err = FfiZakatError {
        code: "JSON_ERROR".to_string(),
        message,
        field: None,
        hint,
        source_label: None,
    };
    err.into()
}

/// Helper to create a serialization error
fn serialization_error(message: String) -> JsValue {
    let err = FfiZakatError {
        code: "SERIALIZATION_ERROR".to_string(),
        message,
        field: None,
        hint: None,
        source_label: None,
    };
    err.into()
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
        .map_err(|e| json_error(format!("Invalid Config JSON: {}", e), Some("Check JSON format".to_string())))?;
        
    let assets: Vec<PortfolioItem> = from_value(assets_json)
        .map_err(|e| json_error(format!("Invalid Assets JSON: {}", e), Some("Check JSON format".to_string())))?;

    let mut portfolio = ZakatPortfolio::new();
    for asset in assets {
        portfolio = portfolio.add(asset);
    }
    
    let result = portfolio.calculate_total(&config);
    
    to_value(&result)
        .map_err(|e| serialization_error(format!("Failed to serialize result: {}", e)))
}

/// Helper: Calculate Zakat for a single asset just like the portfolio but simpler
#[wasm_bindgen]
pub fn calculate_single_asset(config_json: JsValue, asset_json: JsValue) -> Result<JsValue, JsValue> {
    let config: ZakatConfig = from_value(config_json)
        .map_err(|e| json_error(format!("Invalid Config JSON: {}", e), Some("Check JSON format".to_string())))?;
    
    let asset: PortfolioItem = from_value(asset_json)
        .map_err(|e| json_error(format!("Invalid Asset JSON: {}", e), Some("Check JSON format".to_string())))?;

    // ZakatError auto-converts to JsValue via the From impl in types.rs!
    let details = asset.calculate_zakat(&config)?;
        
    to_value(&details)
        .map_err(|e| serialization_error(format!("Failed to serialize result: {}", e)))
}

/// Helper: Test if WASM is alive
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Zakat WASM is ready.", name)
}

