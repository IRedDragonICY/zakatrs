//! Prelude module for ZakatRS
//!
//! This module re-exports commonly used structs, traits, and types to allow
//! for easier usage of the library.
//!
//! # Usage
//!
//! ```rust
//! use zakat::prelude::*;
//! ```

// Core exports
pub use crate::config::ZakatConfig;
pub use crate::madhab::{Madhab, NisabStandard};
pub use crate::portfolio::{ZakatPortfolio, PortfolioResult, PortfolioItemResult};
#[cfg(feature = "async")]
pub use crate::portfolio::AsyncZakatPortfolio;

pub use crate::traits::CalculateZakat;
#[cfg(feature = "async")]
pub use crate::traits::AsyncCalculateZakat;
pub use crate::types::{WealthType, ZakatDetails, ZakatError};
pub use crate::inputs::IntoZakatDecimal;

// Re-export specific calculators and types
pub use crate::builder::AssetBuilder;
pub use crate::maal::business::{BusinessZakat, BusinessZakatBuilder};
pub use crate::config::ZakatConfigBuilder;
pub use crate::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};
pub use crate::maal::investments::{InvestmentAssets, InvestmentType};
pub use crate::maal::precious_metals::PreciousMetals;
pub use crate::maal::agriculture::{AgricultureAssets, IrrigationMethod};
pub use crate::maal::livestock::{LivestockAssets, LivestockType, LivestockPrices};
pub use crate::maal::mining::{MiningAssets, MiningType};
pub use crate::fitrah::calculate_fitrah;
