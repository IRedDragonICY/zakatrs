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
pub use crate::config::{ZakatConfig, Madhab, NisabStandard};
pub use crate::portfolio::{ZakatPortfolio, PortfolioResult};
pub use crate::traits::CalculateZakat;
pub use crate::types::{WealthType, ZakatDetails, ZakatError};

// Re-export specific calculators and types
pub use crate::maal::business::{BusinessAssets, BusinessZakatCalculator};
pub use crate::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};
pub use crate::maal::investments::{InvestmentAssets, InvestmentType};
pub use crate::maal::precious_metals::PreciousMetals;
pub use crate::maal::agriculture::{AgricultureAssets, IrrigationMethod};
pub use crate::maal::livestock::{LivestockAssets, LivestockType, LivestockPrices};
pub use crate::maal::mining::{MiningAssets, MiningType};
pub use crate::fitrah::calculate_fitrah;
