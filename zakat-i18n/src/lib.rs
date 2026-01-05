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

/// Trait for loading translation resources asynchronously.
/// 
/// This allows decoupling the translation files from the binary, enabling
/// lazy loading in environments like WASM (fetching from URL) or mobile (fetching from disk/network).
#[cfg(not(target_arch = "wasm32"))]
pub trait ResourceLoader: Send + Sync + 'static {
    /// Load a Fluent resource string for the given locale.
    fn load_resource(&self, locale: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
}

#[cfg(target_arch = "wasm32")]
pub trait ResourceLoader: 'static {
    /// Load a Fluent resource string for the given locale.
    fn load_resource(&self, locale: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>>>>;
}
