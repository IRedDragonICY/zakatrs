# zakat

A comprehensive, type-safe, and Fiqh-compliant Zakat calculation library for Rust.

## Overview

This crate provides a complete solution for calculating Islamic Zakat obligations across all asset classes. It implements classical Fiqh rules with support for multiple schools of thought (Madhabs).

## Features

- Type-safe calculations using `rust_decimal` for financial precision
- Support for Gold, Silver, Business, Agriculture, Livestock, Income, and Modern Assets
- Madhab-aware calculation strategies (Hanafi, Shafi'i, Maliki, Hanbali)
- Internationalization with RTL language support
- Hawl (lunar year) tracking
- Live metal pricing integration
- Multiple platform targets: Native, WASM, Python, Dart, Kotlin

## Installation

```toml
[dependencies]
zakat = "1.0"
```

## Quick Start

```rust
use zakat::prelude::*;

fn main() -> Result<(), ZakatError> {
    // Configure with Hanafi rules
    let config = ZakatConfig::hanafi()
        .gold_price_per_gram(65.0)
        .silver_price_per_gram(0.80)
        .build()?;

    // Calculate gold zakat
    let gold = PreciousMetals::gold(100.0)
        .purity(24)
        .calculate_zakat(&config)?;

    println!("Zakat due: {}", gold.zakat_due);

    // Portfolio calculation
    let mut portfolio = ZakatPortfolio::new();
    portfolio.add(PreciousMetals::gold(100.0));
    portfolio.add(BusinessZakat::new().cash(50000.0).inventory(25000.0));

    let total = portfolio.calculate_total(&config)?;
    println!("Total Zakat: {}", total.total_zakat);

    Ok(())
}
```

## Asset Types

| Type | Description |
|------|-------------|
| `PreciousMetals` | Gold and silver with purity calculations |
| `BusinessZakat` | Trade goods, cash, receivables, inventory |
| `AgricultureAssets` | Crops with irrigation differentiation |
| `LivestockAssets` | Camels, cattle, sheep with Nisab tables |
| `IncomeZakatCalculator` | Professional income (gross/net) |
| `InvestmentAssets` | Stocks, bonds, cryptocurrency |
| `MiningAssets` | Minerals and Rikaz (buried treasure) |

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `i18n` | Localization support | Yes |
| `ledger` | Hawl tracking and timeline | Yes |
| `providers` | Live price fetching | Yes |
| `async` | Async calculation support | Yes |
| `sqlite` | SQLite persistence | No |
| `wasm` | WebAssembly bindings | No |
| `python` | Python bindings | No |

## Crate Structure

This is a facade crate that re-exports functionality from:

| Crate | Purpose |
|-------|---------|
| `zakat-core` | Core calculations and types |
| `zakat-i18n` | Internationalization |
| `zakat-ledger` | Hawl and timeline tracking |
| `zakat-providers` | Live pricing |
| `zakat-sqlite` | Database persistence |

## Platform Support

| Platform | Status |
|----------|--------|
| Linux / macOS / Windows | Supported |
| WebAssembly (Browser) | Supported |
| Python 3.8+ | Supported |
| Dart / Flutter | Supported |
| Kotlin / Android | Supported |

## Documentation

- [Rust API Docs](https://docs.rs/zakat)
- [Python Usage](https://github.com/IRedDragonICY/zakatrs/blob/main/docs/USAGE_PYTHON.md)
- [JavaScript/WASM Usage](https://github.com/IRedDragonICY/zakatrs/blob/main/docs/USAGE_JS.md)
- [Flutter Usage](https://github.com/IRedDragonICY/zakatrs/blob/main/docs/USAGE_FLUTTER.md)

## License

MIT
