use wasm_bindgen::prelude::*;
use crate::prelude::*;
use crate::config::ZakatConfig;
use crate::portfolio::ZakatPortfolio;
use crate::assets::PortfolioItem;
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
pub fn calculate_portfolio_wasm(config_json: JsValue, assets_json: JsValue) -> Result<JsValue, JsError> {
    let config: ZakatConfig = from_value(config_json)
        .map_err(|e| JsError::new(&format!("Invalid Config JSON: {}", e)))?;
        
    let assets: Vec<PortfolioItem> = from_value(assets_json)
        .map_err(|e| JsError::new(&format!("Invalid Assets JSON: {}", e)))?;

    let mut portfolio = ZakatPortfolio::new();
    for asset in assets {
        portfolio = portfolio.add(asset);
    }
    
    let result = portfolio.calculate_total(&config);
    
    to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization Error: {}", e)))
}

/// Helper: Calculate Zakat for a single asset just like the portfolio but simpler
#[wasm_bindgen]
pub fn calculate_single_asset(config_json: JsValue, asset_json: JsValue) -> Result<JsValue, JsError> {
    let config: ZakatConfig = from_value(config_json)
        .map_err(|e| JsError::new(&format!("Invalid Config JSON: {}", e)))?;
    
    let asset: PortfolioItem = from_value(asset_json)
        .map_err(|e| JsError::new(&format!("Invalid Asset JSON: {}", e)))?;

    let details = asset.calculate_zakat(&config)
        .map_err(|e| JsError::new(&format!("Calculation Error: {}", e)))?;
        
    to_value(&details)
        .map_err(|e| JsError::new(&format!("Serialization Error: {}", e)))
}

/// Helper: Test if WASM is alive
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Zakat WASM is ready.", name)
}
