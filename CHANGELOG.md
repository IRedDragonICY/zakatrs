# Changelog

All notable changes to this project will be documented in this file.

## [0.16.0] - 2025-12-31

### Added
- **Internationalization (i18n) Support**: Added robust i18n support using Project Fluent.
- **New Locales**: Added support for `en-US` (English), `id-ID` (Indonesian), and `ar-SA` (Arabic).
- **Localized Output**: `ZakatDetails` now provides `explain_in(locale)` and `summary_in(locale)` for localized calculation traces.
- **Currency Formatting**: Added `CurrencyFormatter` trait for locale-aware currency display (e.g., `Rp` for ID, `,` vs `.` separators).
- **Localized Warnings**: Validation warnings are now structured for localization.

### Changed
- **CalculationStep API**: Refactored `CalculationStep` to use translation keys instead of hardcoded English strings.
- **Inputs Input**: Refined `sanitize_numeric_string` for professional-grade heuristic parsing of international number formats.

## [0.15.0] - 2025-12-31

### Added
- **Dynamic Trade Goods Rate**: `aggregate_and_summarize` now uses the rate defined in `ZakatStrategy` (e.g., 2.577%) instead of a hardcoded 2.5%.
- **Config Builder**: Added `ZakatConfig::build()` for explicit validation at the end of a configuration chain.
- **Diagnostic Reports**: Enhanced `ZakatError` with `context()` returning structured JSON and improved `report()` output.
- **WASM structured Errors**: WebAssembly functions now return detailed error objects with codes (`INVALID_INPUT`, `CONFIG_ERROR`) instead of plain strings.

### Performance
- **Zero-Copy Sanitization**: Rewrote `sanitize_numeric_string` to use single-pass pre-allocation, significantly reducing memory churn during input parsing.

## [0.14.0] - 2025-12-31

### Added
- **Security Hardening**:
    - **DoS Prevention**: Implemented `MAX_INPUT_LEN` (64 chars) check for all numeric inputs to prevent memory exhaustion attacks.
    - **Robust Sanitization**: Stripped non-breaking spaces (`\u{00A0}`) and invisible control characters from inputs.
    - **Safe Env Loading**: `ZakatConfig::from_env()` now trims whitespace to prevent parsing errors from accidental padding.
- **Async Performance**:
    - **Parallel Execution**: Refactored `calculate_total_async` to use `FuturesUnordered`, allowing concurrent asset calculations (e.g., fetching multiple live prices in parallel).
- **Observability**:
    - **Tracing Integration**: Added `tracing` instrumentation to core portfolio methods (`calculate_total`, `calculate_total_async`).
    - **Validation Logs**: Validation failures and value clamping (e.g., negative net assets) are now logged as `warn!`.
- **Developer Ergonomics**:
    - **Config Layering**: Added `ZakatConfig::merge(self, other)` to support hierarchical configuration (e.g., defaults -> config file -> env vars).

## [0.13.0] - 2025-12-31

### Added
- **Structured Explanation API**: New `ZakatExplanation` struct for API consumers (React, Vue, etc.).
    - Added `to_explanation(&self) -> ZakatExplanation` method to `ZakatDetails`.
    - Refactored `explain()` to use `to_explanation().to_string()`.
- **Aggregated Validation Errors**: New `ZakatError::MultipleErrors(Vec<ZakatError>)` variant.
    - `validate()` now returns all collected errors, not just the first.
    - Updated `with_source()`, `with_asset_id()`, and `report()` to handle `MultipleErrors`.
- **Portfolio Mutability**: Added `get_mut(&mut self, id: Uuid) -> Option<&mut PortfolioItem>` method.
    - Allows in-place modification of portfolio assets without remove/re-add.
- **Explicit Locale Handling**: New `InputLocale` enum and `LocalizedInput` struct.
    - Added `with_locale(val, locale)` helper for unambiguous parsing.
    - Locales: `US` (1,000.00), `EU` (1.000,00), `EasternArabic`.
    - Example: `with_locale("€1.234,50", InputLocale::EU)` → `1234.50`.

### Changed
- **Panic-Free Purity Setter**: `PreciousMetals::purity()` no longer panics on invalid input.
    - Errors are collected in `_input_errors` and surfaced via `validate()` or `calculate_zakat()`.
    - Added `_input_errors: Vec<ZakatError>` field to `PreciousMetals`.

### Fixed
- **Non-Exhaustive Pattern**: Fixed `report()` method to handle `MultipleErrors` variant.

## [0.12.0] - 2025-12-31

### Added
- **Arabic Numeral Support**: Input parsing now handles Eastern Arabic numerals (`٠-٩`) and Perso-Arabic numerals (`۰-۹`).
    - Example: `"١٢٣٤.٥٠"` → `1234.50`
- **Enhanced Error Context**: All `ZakatError` variants now include an optional `asset_id: Option<uuid::Uuid>` field.
    - Added `ZakatError::with_asset_id(uuid)` method for setting the asset ID.
    - Updated `ZakatError::report()` to display the asset ID when present.
- **Input Validation Method**: Added `validate()` method to asset structs using `zakat_asset!` macro.
- **Livestock Optimization**: Early return when `count == 0` skips unnecessary calculations.

### Changed
- **Panic-Free Setters**: Fluent setters in `BusinessZakat`, `MiningAssets`, and macro-generated structs no longer panic on invalid input.
    - Errors are collected and deferred until `validate()` or `calculate_zakat()` is called.
    - *Breaking Change*: Users who relied on immediate panics must now check `validate()` or handle errors from `calculate_zakat()`.
- **Config Partial Loading**: `ZakatConfig` optional fields now use `#[serde(default)]`, allowing partial JSON loading without errors on missing keys.
    - Fields: `rice_price_per_kg`, `rice_price_per_liter`, `cash_nisab_standard`, `nisab_gold_grams`, `nisab_silver_grams`, `nisab_agriculture_kg`.

## [0.11.0] - 2025-12-31

### Added
- **ID Restoration**: Added `with_id(uuid::Uuid)` method to all asset types for database/serialization restoration.
- **Gold Purity Validation**: `PreciousMetals::purity()` now validates that purity is between 1-24 Karat.
- **European Locale Support**: Input parsing now handles European decimal format (e.g., `€12,50` → `12.50`).

### Changed
- **Dynamic Trade Goods Rate**: All calculators now use `config.strategy.get_rules().trade_goods_rate` instead of hardcoded `2.5%`.
    - Affected modules: `business`, `investments`, `income`, `mining`, `precious_metals`.
- **Fail-Fast Setters**: Fluent setters now panic on invalid input instead of silently ignoring errors.
    - *Breaking Change*: Invalid inputs will cause panics rather than defaulting to zero.
    - Maintains DX-friendly fluent API (no `.unwrap()` required by users).

### Fixed
- **100x Financial Error**: Fixed locale-aware parsing bug where `€12,50` was incorrectly parsed as `1250`.
- **400% Asset Inflation**: Fixed purity validation allowing `purity(100)` which inflated gold value by `100/24`.
- **Strategy Pattern Disconnect**: Fixed `trade_goods_rate` from `ZakatStrategy` being ignored.

## [0.10.0] - 2025-12-31

### Added
- **Flexible Configuration Arguments**:
    - The `calculate_zakat` method now accepts arguments implementing `ZakatConfigArgument`.
    - Supported inputs: `&ZakatConfig` (standard), `Option<&ZakatConfig>` (uses default if None), `()` (uses default config).
    - Example: `asset.calculate_zakat(())?` or `asset.calculate_zakat(None)?`.
- **Convenience Method**: Added `.calculate()` method as a shortcut for `.calculate_zakat(())`.

### Changed
- **Trait Definition**: Refactored `CalculateZakat` trait to use a generic config argument `C: ZakatConfigArgument`.
    - *Breaking Change*: Manual implementations of `CalculateZakat` must update their method signature.

## [0.9.0] - 2025-12-31

### Added
- **Robust Input Sanitization**:
    - `IntoZakatDecimal` for `&str` and `String` now automatically sanitizes input.
    - Removes commas (`,`), underscores (`_`), and currency symbols (`$`, `£`, `€`, `¥`).
    - Handles whitespace gracefully (e.g., `"$1,000.00"` -> `1000.00`).
- **Structured Warning System**:
    - Added `warnings` field to `ZakatDetails`.
    - Non-fatal issues (like negative net assets clamped to zero) are now reported in the `warnings` vector.
    - Updated `explain()` output to include a "WARNINGS" section when applicable.

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
