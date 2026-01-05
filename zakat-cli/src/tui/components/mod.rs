//! Component widgets for the TUI.
//!
//! This module contains reusable UI components for building
//! the Zakat CLI terminal user interface.

#[allow(dead_code)]
pub mod asset_card;
pub mod spinner;
pub mod stat_card;

#[allow(unused_imports)]
pub use asset_card::{AssetCard, AssetTypeOption};
pub use spinner::LoadingSpinner;
#[allow(unused_imports)]
pub use stat_card::{InlineStat, StatCard};
