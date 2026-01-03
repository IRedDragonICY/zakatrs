# Changelog

## [1.1.0] - 2026-01-03

### Quality of Life Release

This release introduces 5 high-impact "Quality of Life" features focused on developer experience and real-world usability.

### Added

- **Smart Fuzzy Dates (Hawl Tracking)**:
  - New `FuzzyDate` enum with Islamic month variants: `Ramadan(year)`, `Muharram(year)`, `Shawwal(year)`, `DhulHijjah(year)`, `Unknown`
  - New `AcquisitionDate` enum: `Exact(NaiveDate)` or `Approximate(FuzzyDate)`
  - `HawlTracker::with_fuzzy_acquisition()` for approximate date handling
  - `HawlTracker::from_acquisition_date()` accepts both exact and fuzzy dates
  - Uses ICU Calendar for Hijri-to-Gregorian conversion
  - **Fiqh Safety**: `Unknown` dates return `true` (Hawl satisfied) to err on side of caution

- **Portfolio Snapshots (Audit Logs)**:
  - New `PortfolioSnapshot` struct for serializable audit trails
  - Contains: `timestamp`, `config_snapshot`, `inputs`, `result`, `metadata`
  - `ZakatPortfolio::snapshot(&config, &result)` creates snapshots
  - `PortfolioSnapshot::to_json()` and `from_json()` for persistence
  - `.with_metadata(key, value)` builder for custom audit data

- **Automatic Price Fallbacks**:
  - New `BestEffortPriceProvider<P>` in `zakat-providers`
  - Wraps any `PriceProvider` with static fallback prices
  - Caches last known good prices (`last_known_good: Arc<RwLock<...>>`)
  - Logs warnings when using fallback
  - Supports both native (reqwest) and WASM (gloo-net) targets

- **"Almost Payable" State (Sadaqah Recommendation)**:
  - New `ZakatRecommendation` enum: `Obligatory`, `Recommended`, `None`
  - Added `recommendation` field to `ZakatDetails`
  - Returns `Recommended` when `net_assets >= 90% of Nisab` (encouraging voluntary Sadaqah)
  - **Fiqh Safety**: Never marks as `Obligatory` unless strictly payable

- **Interactive CLI Tool (`zakat-cli`)**:
  - New binary crate: `cargo install zakat-cli` or `cargo run -p zakat-cli`
  - Interactive prompts via `inquire` for asset entry
  - Supports: Business, Gold, Silver, Cash, Investments, Agriculture
  - Pretty-printed results with `tabled` and `colored`
  - Live pricing with `BestEffortPriceProvider` fallback
  - Snapshot saving for audit trails
  - CLI flags: `--offline`, `--gold-price`, `--silver-price`, `--verbose`

### Changed

- **XTask**: Added `zakat-cli` to workspace crates list for publishing
- **Prelude Exports**: Added `HawlTracker`, `AcquisitionDate`, `FuzzyDate`, `ZakatRecommendation`, `PortfolioSnapshot`

### Technical

- **78 Tests Passing**: All unit tests verified (67 zakat-core + 11 zakat-providers)
- **Zero Breaking Changes**: All new features are additive

---

## [1.0.1] - 2026-01-03

### Added
- **Automated TypeScript Type Generation**: Integrated `typeshare` for automatic TypeScript definition generation from Rust types.
  - Types are now auto-generated to `pkg/types.ts` during build.
  - All `Decimal` fields are serialized as `string` for precision.
  - JSDoc comments are generated from Rust doc comments (`///`).
- **Multi-Platform Type Generation**: Extended `xtask build-all` to generate types for multiple platforms:
  - **TypeScript**: `pkg/types.ts` (for NPM, JSR, WASM, Deno)
  - **Kotlin**: `zakat_android/.../Types.kt` (for Android)
  - **Swift**: `zakat_ios/.../ZakatTypes.swift` (for iOS, optional)
- **Typeshare Annotations**: Added `#[typeshare::typeshare]` to core public types:
  - `ZakatConfig`, `NetworkConfig` (config.rs)
  - `NisabStandard`, `Madhab`, `ZakatRules` (madhab.rs)
  - `ZakatDetails`, `ZakatExplanation`, `CalculationStep`, `CalculationTrace` (types.rs)
  - `PortfolioItem`, `CustomAsset` (assets.rs)
  - `WealthType`, `PaymentPayload`, `Operation` (types.rs)

### Changed
- **NetworkConfig**: Changed `binance_api_ip` from `IpAddr` to `String` for FFI compatibility.
- **NetworkConfig**: Changed `timeout_seconds` from `u64` to `u32` for typeshare compatibility.
- **JSR Configuration**: Replaced manual `definitions.ts` with auto-generated `types.ts`.

### Removed
- **Manual Type Definitions**: Deleted `jsr-config/definitions.ts` (replaced by auto-generated `pkg/types.ts`).

---

## [1.0.0] - 2026-01-03

### 🎉 First Stable Release

This is the first stable release of `zakatrs`, marking the library as production-ready.

### Changed
- **Cargo Workspace Architecture**: Major refactor from monolithic crate to modular workspace structure.
  - `zakat-core`: Core mathematical logic, types, inputs, and Maal calculations
  - `zakat-i18n`: Fluent localization with embedded locale files (en-US, id-ID, ar-SA)
  - `zakat-ledger`: Hawl tracking and timeline analysis for wealth history
  - `zakat-providers`: Live pricing providers with Binance API integration
  - `zakat-sqlite`: SQLite persistence layer for ledger storage
  - `zakat`: Thin facade crate re-exporting all public APIs (backwards compatible)

### Added
- **WASM Providers Support**: Live pricing now works on WebAssembly using `gloo-net` instead of `reqwest`
  - Platform-specific implementations: `reqwest` for native, `gloo-net` for WASM
  - `web-time` for WASM-compatible timing in cache logic
  - `#[async_trait(?Send)]` for relaxed WASM trait bounds

### Fixed
- **UUID WASM Randomness**: Added `js` feature to `uuid` for proper WASM random generation
- **Feature Flag Isolation**: WASM-specific code now properly gated behind `feature = "wasm"` only

### Technical
- **Shared Dependencies**: All common dependencies managed via `[workspace.dependencies]`
- **Incremental Builds**: Changes to one crate only rebuild affected crates
- **Parallel Compilation**: Independent crates compile in parallel
- **67 Tests Passing**: All unit and integration tests verified across all platforms

### Platform Support
| Platform | Status |
|----------|--------|
| Native (Windows/Linux/macOS) | ✅ |
| WebAssembly (Browser) | ✅ |
| Python (PyO3) | ✅ |
| Dart/Flutter | ✅ |
| Android (UniFFI) | ✅ |

---

## [0.20.2] - 2026-01-02
### Fixed
- **Publish Workflow**: Fixed JSR and Dart publish scripts to correctly handle duplicate artifacts.
- **Dart Metadata**: Added missing `.pubignore` to allow publishing of `README.md` and `LICENSE` despite being gitignored.

## [0.20.1] - 2026-01-02
### Fixed
- **JSR Compliance**: Resolved "Slow Types" errors.
- **Provider Resilience**: Added resilient Binance provider with IP fallback.

## [0.20.0] - 2026-01-02

### Added
- **Temporal Ledger**: Implemented `Al-Hawl Al-Haqiqi` (True Hawl) tracking.
    - Added `LedgerAsset` and `LedgerEvent` for event-sourced wealth reconstruction.
    - Added `LedgerStore` trait and `JsonFileStore` for async persistence.
- **Fiqh Features**:
    - **Hawl Tracking**: Precise day-count logic for Zakat eligibility (354 days).
    - **Purification (Tathir)**: Support for purifying mixed-income assets (e.g. stocks) before calculation.
- **Pricing Providers**:
    - **Binance Integration**: Added `BinancePriceProvider` with DNS over HTTPS (DoH) bypass for censorship resistance.
    - **Live Pricing**: `Currency` enum and `PriceProvider` trait improvements.
- **Internationalization**:
    - **ICU4X**: Migrated to `ICU4X` 1.5 for robust, locale-aware currency formatting (e.g., proper symbol placement, numbering systems).
- **Strict Validation**:
    - `ZakatConfig` now strictly prevents zero-price silent failures for Gold/Silver.
    - WASM bindings are now panic-safe, returning structured `JsValue` errors.

### Changed
- **Error Handling**:
    - **Review**: Boxed large `ZakatError` variants to optimize stack usage (`clippy::result_large_err`).
    - **Localization**: Refactored `ZakatError` to fully support i18n with translation keys and arguments.
- **Bindings & Integrations**:
    - **Python**: Optimized `to_dict` performance and removed internal `serde_json` overhead.
    - **Kotlin**: Fixed silent failures, added `CalculationError` variant, and switched to `String` inputs for financial precision.
    - **Dart**: Removed `rust_builder` duplication and streamlined workspace integration.
- **Refactoring**:
    - **DRY Macros**: introduced universal `zakat_asset!` macro to eliminate boilerplate across all asset modules.
    - **Monetary Logic**: Centralized monetary asset calculation logic to `src/maal/calculator.rs`.
    - **Deprecation**: Removed legacy `add_*` methods from `ZakatPortfolio`.

### Fixed
- **Build System**:
    - Fixed WASM build errors by splitting and gating `tokio` features.
    - Fixed Kotlin binding silent errors.
- **Logic**: Unified liability logic in `BusinessZakat` (removed `short_term_liabilities`).

## [0.19.0] - 2026-01-01

### Changed
- **Global Translation State Refactor**: Significant architectural overhaul to remove global static state.
    - **Removed**: Global `TRANSLATOR` static from `src/i18n.rs`.
    - **Added**: `translator` field to `ZakatConfig`, making it the owner of translation state.
    - **Updated**: `ZakatDetails::explain()` and related methods now require `&Translator` to be passed explicitly.
    - **API**: Added `default_translator()` helper and `config.explain()` convenience method.
- **Zakat Dart Refactor**: Major architectural update to `zakat_dart` integration.
    - **Workspace**: Moved `zakat_dart/rust` to root cargo workspace for better dependency management.
    - **Type Safety**: Replaced unsafe `String`/`f64` passing with strict `Decimal` types across the FFI boundary using `FrbDecimal`.
    - **Dart Extensions**: Implemented `Decimal <-> FrbDecimal` conversion extensions for seamless DX.


## [0.18.0] - 2026-01-01
### Added
- **Validation Trait**: Exposed `is_valid()` and `validate_input()` methods in `CalculateZakat` trait.
    - Allows checking for validity (e.g., negative inputs) without strictly running the full calculation.
- **Asset Validation**: Implemented `validate_input()` for all asset types (`Business`, `PreciousMetals`, `Livestock`, etc.).

### Fixed
- **Trait Bounds**: Added `Serialize`/`Deserialize` to `ZakatLocale`, enabling proper serialization chains.
- **Config Usage**: Fixed internal compilation error in `LivestockAssets` config resolution.

## [0.17.2] - 2026-01-01
### Fixed
- **NPM/JSR Metadata**: Fixed automated build to unconditionally sync `README.md` and metadata to `pkg/` directory, ensuring NPM/JSR pages are up-to-date even if WASM build is cached/skipped.

## [0.17.1] - 2026-01-01
### Fixed
- **Pub.dev Metadata**: Updated `repository` URL to correctly point to the `zakat_dart` subdirectory, fixing package verification scores.

## [0.17.0] - 2025-12-31

### Added
- **Panic-Free Setters (Complete)**: Extended deferred error handling to `PreciousMetals`, `InvestmentAssets`, and `IncomeZakatCalculator`.
    - Setters like `weight()`, `debt()`, `income()`, `value()` no longer panic on invalid input.
    - Errors are deferred and reported via `validate()` or `calculate_zakat()`.
- **Validation**: Added `validate()` method to `InvestmentAssets` and `IncomeZakatCalculator`.
- **NPM Publication**: Published as `@islamic/zakat` on NPM with full WebAssembly support.
    - **WASM bindings**: `src/wasm.rs` exposes `calculate_portfolio_wasm` and `calculate_single_asset` for JS consumers.
    - **Hybrid Build**: Configured for both Node.js and Browser environments via `wasm-pack`.
    - **Public Access**: Scoped package `@islamic` is configured for public access.

### Fixed
- **Trace Output**: Fixed deserialization of `CalculationStep` in tests.
- **Explain Output**: Aligned `explain()` output format in tests.
- **Added:** `zakat_dart` Flutter package for mobile/desktop apps, using `flutter_rust_bridge`.
- **Added:** Official JSR support (`@islam/zakat`) with automated build scripts.

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
