# Changelog

All notable changes to this project will be documented in this file.

## [0.8.0] - 2025-12-31

### Added
- **Semantic Constructors**: Introduced explicit, type-safe constructors for better DX:
    - `BusinessZakat::cash_only(amount)`
    - `PreciousMetals::gold(weight)`, `PreciousMetals::silver(weight)`
    - `IncomeZakatCalculator::from_salary(amount)`
    - `InvestmentAssets::stock(value)`, `InvestmentAssets::crypto(value)`
- **Configuration Presets**: Added `ZakatConfig::hanafi()` and `ZakatConfig::shafi()` helper methods.
- **Unified Error Reporting**: Added `ZakatError::report()` for standardized diagnostics.
- **WASM Support**: Added `wasm` feature flag and `src/wasm.rs` facade for WebAssembly compatibility.
- **Safe Math Wrappers**: Implemented checked arithmetic for all Decimal operations to prevent panics.

### Changed
- **Direct Numeric Literals**: The API now supports direct `f64` literals (e.g., `0.025`) using `IntoZakatDecimal`.
- **Internal Optimization**: Refactored internal library code (`src/`) to use `dec!` macro for compile-time precision.
- **Portfolio API**: Deprecated closure-based `add_*` methods in favor of the generic `.add()`.
- **Refactor**: Replaced `Decimal::new` with `dec!` in internal logic and test assertions.

### Fixed
- **BusinessZakat ID**: Fixed recursion stack overflow in `get_id()`.
- **Warnings**: Resolved unused import warnings across the codebase.

## [0.7.0] - 2025-12-30

### Added
- **Serialization**: Added `serde` support for `PortfolioItem` enum, allowing full JSON save/load of Portfolios.
- **PortfolioItem Enum**: Unified asset storage in Portfolio to a single enum for better type safety and serialization.

### Changed
- **Doc Audit**: Comprehensive review and cleanup of all documentation comments.

## [0.6.1] - 2025-12-30

### Fixed
- **Error Handling**: Improved error precision for Livestock calculations.
- **Financial Precision**: Enhanced rounding logic for monetary assets.

## [0.6.0] - 2025-12-30

### Added
- **Fiqh Compliance Audit**: Validated logic against classical Fiqh sources.
- **Dynamic Portfolio**: Added `add_with_id`, `replace`, and `remove` methods using stable UUIDs.

## [0.5.0] - 2025-12-30

### Changed
- **Fluent Struct API**: Complete migration from Builder Pattern to Fluent Structs (e.g., `BusinessZakat::new().cash(...)`).
- **Validation**: Moved validation to `calculate_zakat()` time rather than build time.

## [0.4.1] - 2025-12-30

### Added
- **Async Documentation**: Updated README with async usage examples.
- **Dependency Updates**: Bumped internal dependencies.

## [0.4.0] - 2025-12-30

### Changed
- **Business Zakat API**: Refactored `BusinessZakat` to be more ergonomic.
- **Validation Hardening**: Stricter checks for negative values in business assets.

## [0.3.0] - 2025-12-29

### Added
- **Portfolio Resilience**: Logic to handle partial failures in portfolio calculations.
- **Unified Builder Pattern**: Standardized builder implementation across all assets.

## [0.2.0] - 2025-12-29

### Added
- **Strategy Pattern**: Introduced `ZakatStrategy` trait for pluggable calculation rules (Madhabs).
- **Type Safety**: Enhanced type usage for better compile-time guarantees.
- **Utils**: Added utility functions for common Zakat math.

## [0.1.5] - 2025-12-29

### Added
- **Livestock Reporting**: Detailed breakage of "In-Kind" payments (e.g., "1 Bint Makhad").
- **Config DX**: Improved configuration ergonomics.

## [0.1.4] - 2025-12-29

### Added
- **Asset Labeling**: Added `.label("My Asset")` support for better debugging.
- **Input Sanitization**: Basic blocking of invalid negative inputs where sensible.

## [0.1.3] - 2025-12-29

### Added
- **Madhab Presets**: Preliminary support for Madhab-based rules.
- **Hawl Logic**: Validated 1-year holding period logic.

## [0.1.0] - 2025-12-24

### Added
- **Initial Release**: Core support for Gold, Silver, Business, Agriculture, Livestock, Mining, and Income Zakat.
- **Optimizations**: O(1) algorithms for Livestock calculations.
