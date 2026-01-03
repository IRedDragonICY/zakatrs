/// Flutter Rust Bridge API modules.
///
/// This module exposes the Dart FFI API for the Zakat library.

// Shared types (FrbDecimal, DartZakatConfig, DartZakatResult)
pub mod types;

// Auto-generated asset wrappers (DartBusiness, DartPreciousMetals, etc.)
pub mod assets;

// Portfolio and manager (stateful aggregation)
pub mod zakat;
