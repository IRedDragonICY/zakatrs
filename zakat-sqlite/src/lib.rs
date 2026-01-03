//! zakat-sqlite - SQLite Persistence for Zakat Ledger
//!
//! This crate provides SQLite-based persistence for ledger events
//! using the sqlx async database driver.
//!
//! ## Features
//!
//! - Production-ready SQLite storage
//! - Connection pooling via sqlx
//! - Automatic schema migrations
//! - JSON file-based fallback store

mod persistence;
mod sqlite;

pub use persistence::*;
pub use sqlite::*;
