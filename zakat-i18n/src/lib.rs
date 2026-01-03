//! zakat-i18n - Internationalization and Localization for Zakat Library
//!
//! This crate provides translation and currency formatting capabilities
//! for the Zakat calculation library using fluent-rs and ICU4X.
//!
//! ## Features
//! 
//! - Multi-locale support (English, Indonesian, Arabic)
//! - Currency formatting using ICU4X
//! - Fluent translation bundles for error messages and UI strings

mod i18n;

pub use i18n::*;
