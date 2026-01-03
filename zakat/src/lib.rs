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
