//! # Zakat Library - Facade Crate
//!
//! This crate re-exports all functionality from the zakat workspace crates
//! for backward compatibility and convenience.
//!
//! ## Workspace Crates
//!
//! - [`zakat_core`] - Core types, traits, and calculators
//! - [`zakat_i18n`] - Internationalization and localization (feature: `i18n`)
//! - [`zakat_ledger`] - Event sourcing and Hawl tracking (feature: `ledger`)
//! - [`zakat_providers`] - Live price fetching (feature: `providers`)
//! - [`zakat_sqlite`] - SQLite persistence (feature: `sqlite`)
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use zakat::prelude::*;
//!
//! let gold = PreciousMetals::gold(100.0)?;
//! let config = ZakatConfig::default().with_gold_price(65.0)?;
//! let result = gold.calculate_zakat(&config)?;
//! println!("Zakat due: {}", result.zakat_due);
//! ```

// Re-export core crate
pub use zakat_core::*;

// Re-export i18n crate
#[cfg(feature = "i18n")]
pub mod i18n {
    pub use zakat_i18n::*;
}

// Re-export ledger crate
#[cfg(feature = "ledger")]
pub mod ledger {
    pub use zakat_ledger::*;
}

// Re-export providers crate
#[cfg(feature = "providers")]
pub mod pricing {
    pub use zakat_providers::*;
}

// Re-export sqlite crate
#[cfg(feature = "sqlite")]
pub mod sqlite {
    pub use zakat_sqlite::*;
}

// Extended PortfolioItem that includes LedgerAsset
// This provides backward compatibility for users who used the Ledger variant
#[cfg(feature = "ledger")]
mod extended {
    use serde::{Serialize, Deserialize};
    use zakat_core::traits::CalculateZakat;
    
    /// Extended portfolio item that includes ledger assets.
    /// 
    /// This enum provides backward compatibility with the original monolithic crate
    /// where PortfolioItem included a Ledger variant.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum ExtendedPortfolioItem {
        // Re-wrap core items
        Business(zakat_core::maal::business::BusinessZakat),
        Income(zakat_core::maal::income::IncomeZakatCalculator),
        Livestock(zakat_core::maal::livestock::LivestockAssets),
        Agriculture(zakat_core::maal::agriculture::AgricultureAssets),
        Investment(zakat_core::maal::investments::InvestmentAssets),
        Mining(zakat_core::maal::mining::MiningAssets),
        PreciousMetals(zakat_core::maal::precious_metals::PreciousMetals),
        Fitrah(zakat_core::fitrah::FitrahCalculator),
        Custom(zakat_core::assets::CustomAsset),
        // Add ledger asset
        Ledger(zakat_ledger::LedgerAsset),
    }

    impl CalculateZakat for ExtendedPortfolioItem {
        fn calculate_zakat<C: zakat_core::traits::ZakatConfigArgument>(&self, config: C) -> Result<zakat_core::types::ZakatDetails, zakat_core::types::ZakatError> {
            match self {
                ExtendedPortfolioItem::Business(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Income(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Livestock(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Agriculture(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Investment(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Mining(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::PreciousMetals(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Fitrah(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Custom(asset) => CalculateZakat::calculate_zakat(asset, config),
                ExtendedPortfolioItem::Ledger(asset) => CalculateZakat::calculate_zakat(asset, config),
            }
        }

        fn get_label(&self) -> Option<String> {
            match self {
                ExtendedPortfolioItem::Business(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Income(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Livestock(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Agriculture(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Investment(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Mining(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::PreciousMetals(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Fitrah(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Custom(asset) => CalculateZakat::get_label(asset),
                ExtendedPortfolioItem::Ledger(asset) => CalculateZakat::get_label(asset),
            }
        }

        fn get_id(&self) -> uuid::Uuid {
            match self {
                ExtendedPortfolioItem::Business(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Income(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Livestock(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Agriculture(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Investment(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Mining(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::PreciousMetals(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Fitrah(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Custom(asset) => CalculateZakat::get_id(asset),
                ExtendedPortfolioItem::Ledger(asset) => CalculateZakat::get_id(asset),
            }
        }
    }

    // Conversions from individual types to ExtendedPortfolioItem
    impl From<zakat_ledger::LedgerAsset> for ExtendedPortfolioItem {
        fn from(asset: zakat_ledger::LedgerAsset) -> Self {
            ExtendedPortfolioItem::Ledger(asset)
        }
    }

    impl From<zakat_core::assets::PortfolioItem> for ExtendedPortfolioItem {
        fn from(item: zakat_core::assets::PortfolioItem) -> Self {
            match item {
                zakat_core::assets::PortfolioItem::Business(a) => ExtendedPortfolioItem::Business(a),
                zakat_core::assets::PortfolioItem::Income(a) => ExtendedPortfolioItem::Income(a),
                zakat_core::assets::PortfolioItem::Livestock(a) => ExtendedPortfolioItem::Livestock(a),
                zakat_core::assets::PortfolioItem::Agriculture(a) => ExtendedPortfolioItem::Agriculture(a),
                zakat_core::assets::PortfolioItem::Investment(a) => ExtendedPortfolioItem::Investment(a),
                zakat_core::assets::PortfolioItem::Mining(a) => ExtendedPortfolioItem::Mining(a),
                zakat_core::assets::PortfolioItem::PreciousMetals(a) => ExtendedPortfolioItem::PreciousMetals(a),
                zakat_core::assets::PortfolioItem::Fitrah(a) => ExtendedPortfolioItem::Fitrah(a),
                zakat_core::assets::PortfolioItem::Custom(a) => ExtendedPortfolioItem::Custom(a),
            }
        }
    }
}

#[cfg(feature = "ledger")]
pub use extended::ExtendedPortfolioItem;

// Re-exports for convenience
pub use zakat_core::prelude;
pub use zakat_core::ZakatConfig;
pub use zakat_core::traits::CalculateZakat;
pub use zakat_core::types::{ZakatDetails, ZakatError, WealthType};
pub use zakat_core::portfolio::ZakatPortfolio;
pub use zakat_core::assets::PortfolioItem;
pub use zakat_core::madhab::{ZakatStrategy, ZakatRules};
pub use zakat_core::inputs::{IntoZakatDecimal, InputLocale, LocalizedInput, with_locale};

// Re-export i18n types
#[cfg(feature = "i18n")]
pub use zakat_i18n::{ZakatLocale, Translator, CurrencyFormatter, default_translator};

// Re-export ledger types
#[cfg(feature = "ledger")]
pub use zakat_ledger::{LedgerAsset, LedgerEvent, HawlTracker};

// Re-export pricing types  
#[cfg(feature = "providers")]
pub use zakat_providers::{Prices, StaticPriceProvider, PriceProvider, CachedPriceProvider};

// Re-export sqlite types
#[cfg(feature = "sqlite")]
pub use zakat_sqlite::{SqliteStore, LedgerStore, JsonFileStore};


// WASM Helper for compatibility with test suite
#[cfg(feature = "wasm")]
pub mod wasm_helpers {

    use wasm_bindgen::prelude::*;
    use serde::{Deserialize, Serialize};
    use rust_decimal::Decimal;




    fn to_camel_case(s: &str) -> String {
        let mut result = String::new();
        let mut next_cap = false;
        for c in s.chars() {
            if c == '_' {
                next_cap = true;
            } else {
                if next_cap {
                    result.push(c.to_ascii_uppercase());
                    next_cap = false;
                } else {
                    result.push(c);
                }
            }
        }
        result
    }

    fn preprocess_input(val: serde_json::Value) -> serde_json::Value {
        match val {
            serde_json::Value::Object(map) => {
                let mut new_map = serde_json::Map::new();
                for (k, v) in map {
                    let new_key = to_camel_case(&k);
                    let new_val = if k == "purity" {
                         // Convert string "24" to number 24
                         if let serde_json::Value::String(s) = &v {
                             if let Ok(n) = s.parse::<u64>() {
                                 serde_json::Value::Number(serde_json::Number::from(n))
                             } else {
                                 v.clone()
                             }
                         } else {
                             v.clone()
                         }
                    } else if k == "usage" {
                        if let serde_json::Value::String(s) = &v {
                            if s == "personal_use" {
                                serde_json::Value::String("personalUse".to_string())
                            } else {
                                v.clone()
                            }
                        } else {
                            v.clone()
                        }
                    } else {
                        preprocess_input(v)
                    };
                    new_map.insert(new_key, new_val);
                }
                serde_json::Value::Object(new_map)
            },
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(preprocess_input).collect())
            },
            _ => val
        }
    }

    #[derive(Deserialize)]
    struct WasmRequest {

        asset_type: String,
        config: WasmTestConfig,
        input: serde_json::Value,
    }

    #[derive(Deserialize)]
    struct WasmTestConfig {
        gold_price_per_gram: String,
        silver_price_per_gram: String,
        madhab: Option<String>,
        currency_code: Option<String>,
    }

    impl WasmTestConfig {
        fn to_core(&self) -> zakat_core::config::ZakatConfig {
             use std::str::FromStr;
             let gold = Decimal::from_str(&self.gold_price_per_gram).unwrap_or(Decimal::ZERO);
             let silver = Decimal::from_str(&self.silver_price_per_gram).unwrap_or(Decimal::ZERO);
             
             let mut config = zakat_core::config::ZakatConfig::default()
                 .with_gold_price(gold)
                 .with_silver_price(silver);
             
             if let Some(code) = &self.currency_code {
                 config = config.with_currency_code(code);
             }

             if let Some(m_str) = &self.madhab {
                 match m_str.to_lowercase().as_str() {
                     "hanafi" => config = config.with_madhab(zakat_core::madhab::Madhab::Hanafi),
                     "shafi" => config = config.with_madhab(zakat_core::madhab::Madhab::Shafi),
                     "maliki" => config = config.with_madhab(zakat_core::madhab::Madhab::Maliki),
                     "hanbali" => config = config.with_madhab(zakat_core::madhab::Madhab::Hanbali),
                     _ => {} 
                 }
             }
             config
        }
    }

    #[derive(Serialize)]
    struct WasmResponse {
        is_payable: bool,
        zakat_due: Decimal,
        #[serde(skip_serializing_if = "Option::is_none")]
        error_code: Option<String>,
    }

    #[wasm_bindgen]
    pub fn calculate_single_asset(val: JsValue) -> Result<JsValue, JsValue> {
        let req: WasmRequest = match serde_wasm_bindgen::from_value(val) {
            Ok(r) => r,
            Err(e) => {
                return Err(JsValue::from_str(&format!("Deserialization error: {}", e)));
            }
        };
        let config = req.config.to_core();
        let input_processed = preprocess_input(req.input);

        let result = match req.asset_type.as_str() {
            "business" => {
                let asset_res: Result<zakat_core::maal::business::BusinessZakat, _> = serde_json::from_value(input_processed);
                match asset_res {
                    Ok(asset) => {
                        use zakat_core::traits::CalculateZakat;
                        asset.calculate_zakat(&config)
                    },
                    Err(e) => Err(zakat_core::types::ZakatError::InvalidInput(Box::new(zakat_core::types::InvalidInputDetails {
                        code: zakat_core::types::ZakatErrorCode::InvalidInput,
                        field: "input".to_string(), value: e.to_string(), reason_key: "parse_error".to_string(),
                        ..Default::default()
                    })))
                }
            },
            "gold" | "silver" => {
                let asset_res: Result<zakat_core::maal::precious_metals::PreciousMetals, _> = serde_json::from_value(input_processed);
                match asset_res {
                    Ok(mut asset) => {
                        use zakat_core::traits::CalculateZakat;
                        // Inject metal_type
                        if req.asset_type == "gold" {
                            asset.metal_type = Some(zakat_core::types::WealthType::Gold);
                        } else if req.asset_type == "silver" {
                            asset.metal_type = Some(zakat_core::types::WealthType::Silver);
                        }
                        asset.calculate_zakat(&config)
                    },
                    Err(e) => Err(zakat_core::types::ZakatError::InvalidInput(Box::new(zakat_core::types::InvalidInputDetails {
                        code: zakat_core::types::ZakatErrorCode::InvalidInput,
                        field: "input".to_string(), value: e.to_string(), reason_key: "parse_error".to_string(),
                        ..Default::default()
                    })))
                }
            },
            _ => {
                return Err(JsValue::from_str(&format!("Unknown asset_type: {}", req.asset_type)));
            }
        };

        match result {
            Ok(details) => {
                 let response = WasmResponse {
                     is_payable: details.is_payable,
                     zakat_due: details.zakat_due,
                     error_code: None,
                 };
                 Ok(serde_wasm_bindgen::to_value(&response)?)
            },
            Err(e) => {
                Err(JsValue::from_str(&e.to_string()))
            }
        }
    }
    
    // Simple greet for test-npm.js
    #[wasm_bindgen]
    pub fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }
}
