//! zakat-ledger - Event Sourcing and Hawl Tracking for Zakat
//!
//! This crate provides event-sourced ledger capabilities for tracking wealth
//! over time and determining Hawl (lunar year holding period) satisfaction.
//!
//! ## Features
//!
//! - Event sourcing for financial transactions
//! - Timeline simulation with historical price data
//! - Hawl analysis and tracking
//! - Support for ledger-based Zakat calculations

pub mod events;
pub mod pricing;
pub mod timeline;
pub mod analyzer;
pub mod assets;
pub mod hawl;
pub mod qada;

// Re-exports for convenience
pub use events::{LedgerEvent, TransactionType, EventStream};
pub use pricing::{HistoricalPriceProvider, InMemoryPriceHistory};
pub use timeline::{DailyBalance, simulate_timeline};
pub use analyzer::{LedgerZakatResult, analyze_hawl};
pub use assets::LedgerAsset;
pub use hawl::HawlTracker;
pub use qada::{QadaCalculator, QadaYearResult, QadaReport};
