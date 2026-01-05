//! # Zakat Core
//!
//! Core mathematical logic, data structures, Fiqh rules, and input validation for Zakat calculations.
//!
//! This crate provides the foundational types and traits used across the Zakat library,
//! without any heavy I/O dependencies like networking, databases, or complex i18n.

#[macro_use]
pub mod macros;

pub mod config;
pub mod inputs;
pub mod madhab;
pub mod math;
pub mod maal;
pub mod traits;
pub mod types;
pub mod utils;
pub mod fitrah;
pub mod portfolio;
pub mod assets;
pub mod hawl;
pub mod validation;

pub mod prelude;

// Re-export key types at crate root
pub use config::ZakatConfig;
pub use inputs::{IntoZakatDecimal, InputLocale, LocalizedInput, with_locale};
pub use madhab::{Madhab, NisabStandard, ZakatRules, ZakatStrategy};
pub use traits::{CalculateZakat, ZakatConfigArgument};
pub use types::{WealthType, ZakatDetails, ZakatError, ZakatExplanation};

#[cfg(feature = "async")]
pub use traits::AsyncCalculateZakat;

#[cfg(feature = "python")]
pub mod python;

#[cfg(feature = "uniffi")]
pub mod kotlin;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();




