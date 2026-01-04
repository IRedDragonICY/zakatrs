//! zakat-providers - Live Price Fetching for Zakat
//!
//! This crate provides asynchronous price providers for fetching
//! live gold and silver prices from various APIs.
//!
//! ## Features
//!
//! - `live-pricing` - Enable live API providers (e.g., Binance)
//! - `force-dns-bypass` - Force use of hardcoded IPs for restricted networks

mod pricing;
mod chain;
mod fs_cache;

pub use pricing::*;
pub use chain::*;
pub use fs_cache::*;
