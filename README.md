<div align="center">
<h1>بِسْمِ اللهِ الرَّحْمٰنِ الرَّحِيْمِ</h1>
<h1>السلام عليكم</h1>
</div>

```text
███████╗ █████╗ ██╗  ██╗ █████╗ ████████╗
╚══███╔╝██╔══██╗██║ ██╔╝██╔══██╗╚══██╔══╝
  ███╔╝ ███████║█████╔╝ ███████║   ██║   
 ███╔╝  ██╔══██║██╔═██╗ ██╔══██║   ██║   
███████╗██║  ██║██║  ██╗██║  ██║   ██║   
╚══════╝╚═╝  ╚═╝╚╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   
                                         
       P U R I F Y  Y O U R  W E A L T H 
```

> *"Take from their wealth a charity by which you purify them and cause them increase..."*  
> — **Surah At-Tawbah 9:103**

# Zakat

[![Crates.io](https://img.shields.io/crates/v/zakat.svg)](https://crates.io/crates/zakat)
[![PyPI](https://img.shields.io/pypi/v/zakatrs.svg)](https://pypi.org/project/zakatrs/)
[![NPM](https://img.shields.io/npm/v/@islamic/zakat.svg)](https://www.npmjs.com/package/@islamic/zakat)
[![JSR](https://jsr.io/badges/@islam/zakat)](https://jsr.io/@islam/zakat)
[![Pub Version](https://img.shields.io/pub/v/zakat.svg)](https://pub.dev/packages/zakat)
[![Documentation](https://docs.rs/zakat/badge.svg)](https://docs.rs/zakat)
![License](https://img.shields.io/crates/l/zakat.svg)

Rust library for Islamic Zakat calculation. Uses `rust_decimal` for precision.

## Features

- Gold, Silver, Business, Agriculture, Livestock, Mining & Rikaz
- Stocks, Mutual Funds, Crypto (as liquid assets)
- Professional Income (Gross/Net)
- Zakat Fitrah
- Configurable Nisab thresholds
- Portfolio aggregation (Dam' al-Amwal)
- Dynamic Portfolio (Add, Remove, Replace assets with stable UUIDs)
- Asset Labeling (e.g., "Main Store", "Crypto Wallet")
- Input Sanitization & Validation (Rejects negative values, ensures safe configuration)
- Arabic Numeral Support (Eastern Arabic ٠-٩, Perso-Arabic ۰-۹)
- Flexible Configuration (Env Vars, JSON, Fluent API, Partial Loading)

- Fiqh Compliance (Jewelry exemptions, Madhab-specific rules, Hawl requirements)
- **Hawl (Lunar Year) Tracking** (Precise date-based eligibility)
- **Stock Purification (Tathir)** (Deduct impure income before calculation)
- Async Support (Optional integration with `tokio` and `async-trait`)
- Live Pricing Interface (e.g. for API integration)
- Detailed Reporting (Livestock in-kind details, calculation traces, metadata support)
- `explain()` Debugging (Get human-readable trace of calculations)
- `to_explanation()` Structured API (Get structured data for frontend rendering)
- `with_locale()` Input Parsing (Explicit EU/US/Arabic locale handling)
- Custom Strategies (Pluggable `ZakatStrategy` trait for custom rules)
- Full Serialization (Save/Load Portfolios via `serde` & JSON)

## Internationalization (i18n)

`zakat` now fully supports internationalization for calculation summaries and explanations.

### Supported Locales
- **English (US)**: `ZakatLocale::EnUS` (Default)
- **Indonesian**: `ZakatLocale::IdID`
- **Arabic (Saudi)**: `ZakatLocale::ArSA`

## Use Cases & Documentation

We support multiple platforms. Please refer to the specific documentation for your needs:

- **🦀 [Rust Usage Guide](docs/USAGE_RUST.md)**: Full API documentation for Rust developers.
- **🐍 [Python Usage Guide](docs/USAGE_PYTHON.md)**: Using `zakatrs` bindings.
- **📦 [JavaScript / Node.js Usage](docs/USAGE_JS.md)**: Using `@islamic/zakat` WASM package.
- **💙 [Flutter / Dart Usage](docs/USAGE_FLUTTER.md)**: Using the `zakat` package.

## Installation

### Rust
With Async Support (Default):
```toml
[dependencies]
zakat = "0.20.2"
rust_decimal = "1.39"
tokio = { version = "1", features = ["full"] }
```

Synchronous Only:
```toml
[dependencies]
zakat = { version = "0.20.2", default-features = false }
rust_decimal = "1.39"
```

### Python
```bash
pip install zakatrs
```

### Node.js / Web
**NPM:**
```bash
npm install @islamic/zakat
```

**JSR:**
```bash
npx jsr add @islam/zakat
```

### Flutter (Dart)
```bash
dart pub add zakat
```

## Build & Development

The project includes a comprehensive set of cross-platform scripts (PowerShell for Windows, Bash for Linux/macOS) in the `scripts/` directory to automate common tasks.

| Task | Script (Windows) | Script (Linux/Mac) | Description |
| :--- | :--- | :--- | :--- |
| **Build All** | `scripts/build-all.ps1` | `scripts/build-all.sh` | Builds Rust Core, WASM bindings, and Dart bindings. |
| **Build Core** | `cargo build` | `cargo build` | Standard Rust build. |
| **Build WASM** | `scripts/build-wasm.ps1` | `scripts/build-wasm.sh` | Compiles to WASM using `wasm-pack` and generates NPM package in `pkg/`. |
| **Build Dart** | `scripts/build-dart.ps1` | `scripts/build-dart.sh` | Generates Flutter Rust Bridge bindings and compiles the Dart library. |
| **Sync Versions** | `scripts/sync-versions.ps1` | `scripts/sync-versions.sh` | Synchronizes version numbers across `Cargo.toml`, `package.json`, and `pubspec.yaml` (coming soon). |
| **Publish** | `scripts/publish-all.ps1` | `scripts/publish-all.sh` | Automates publishing to Crates.io, NPM, and JSR (requires auth tokens). |

> **Note**: For Windows users, run scripts from PowerShell. For Linux/Mac, ensure scripts are executable (`chmod +x scripts/*.sh`).

## Modules

| Module | Nisab |
| :--- | :--- |
| `maal::precious_metals` | 85g Gold / 595g Silver |
| `maal::business` | 85g Gold |
| `maal::income` | 85g Gold |
| `maal::investments` | 85g Gold |
| `maal::agriculture` | 653 kg |
| `maal::livestock` | Count-based |
| `maal::mining` | Rikaz: None / Mines: 85g Gold |
| `fitrah` | N/A |

## Contributing

1. Add tests
2. Use `rust_decimal`
3. If adding async features, ensure they are gated behind `#[cfg(feature = "async")]`
4. Run `cargo test` and `cargo check --no-default-features`

## Support

<div align="center">

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-%E2%9D%A4-ea4aaa?style=for-the-badge&logo=github-sponsors)](https://github.com/sponsors/IRedDragonICY)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/ireddragonicy)
[![Patreon](https://img.shields.io/badge/Patreon-F96854?style=for-the-badge&logo=patreon&logoColor=white)](https://patreon.com/ireddragonicy)
[![PayPal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://paypal.com/paypalme/IRedDragonICY)
[![Saweria](https://img.shields.io/badge/Saweria-F4801C?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSIjZmZmZmZmIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSI4Ii8+PC9zdmc+)](https://saweria.co/IRedDragonICY)

</div>


## License

[MIT](LICENSE)

## Fiqh References & Compliance

We built this library with a deep respect for Islamic Jurisprudence (Fiqh), ensuring that every calculation isn't just mathematically correct, but religiously valid. Here is the breakdown of the scholarly sources (Dalil) and methodlologies we rely on.

### 1. Precious Metals (Gold & Silver)
For gold and silver, we adhere to the standard Nisab thresholds established in the Sunnah: **85 grams for Gold** (20 Dinars) and **595 grams for Silver** (200 Dirhams). This is based on the Hadith of Ali (ra) in *[Sunan Abu Dawud 1573](https://sunnah.com/abudawud:1573)* and *[Sahih Muslim 979](https://sunnah.com/muslim:979)*.

**What about Jewelry?**
This is a classic area of valid difference (*Ikhtilaf*). By default, we follow the **Hanafi view** (as found in *Al-Hidayah*) which regards personal jewelry as Zakatable wealth, citing the Hadith of the bracelets of fire (*[Sunan Abu Dawud 1558](https://sunnah.com/abudawud:1558)*). However, we fully support the majority opinion (**Shafi'i, Maliki, Hanbali**) which treats permissible personal jewelry as exempt. You can easily toggle this in your configuration to match your Madhab.

### 2. Business Assets (Urud al-Tijarah)
All trade goods are valued at their **current market price**, not the cost price. This comes from the Prophet's (ﷺ) command to Samurah bin Jundub to pay Zakat on what is "prepared for sale" (*[Sunan Abu Dawud 1562](https://sunnah.com/abudawud:1562)*).

Regarding debts: We deduct **Immediate Liabilities** (*Dayn al-Hal*) before calculation. This prevents you from paying Zakat on money that essentially belongs to your creditors, a principle supported by classical texts like *Al-Mughni* and modern standards like **[AAOIFI No. 35](https://aaoifi.com)**.

### 3. Agriculture (Zuru' wal-Thimar)
We handle the varying rates for crops based on how much effort goes into watering them:
*   **10% (Ushr)** for crops watered naturally by rain (*[Sahih Bukhari 1483](https://sunnah.com/bukhari:1483)*).
*   **5% (Half-Ushr)** for strictly irrigated crops requiring cost and labor (*[Sahih Muslim 981](https://sunnah.com/muslim:981)*).

For the Nisab (5 Awsuq), we use the standard conversion of approximately **653 kg** (derived from *[Sahih Muslim 979](https://sunnah.com/muslim:979)*), following the research of Dr. Yusuf Al-Qaradawi in his monumental work, *Fiqh al-Zakah*.

### 4. Livestock (An'am)
Our camel Zakat calculator is meticulously coded to follow the exact age tiers (like *Bint Makhad*, *Bint Labun*) outlined in the famous **Letter of Abu Bakr (ra)** preserved in *[Sahih Bukhari 1454](https://sunnah.com/bukhari:1454)*.

Crucially, we enforce the **Saimah** condition: Zakat is only due if your livestock grazes freely on open pastures for the majority of the year. Feed-lot animals are generally exempt from this specific category.

### 5. Modern Assets & Portfolio
*   **Stocks & Crypto**: We treat these as Trade Goods (*Urud*), subjecting them to a 2.5% rate on their market value, consistent with **[AAOIFI Standard 35](https://aaoifi.com)**. We also support **Purification (Tathir)**, allowing you to automatically deduct percentages of non-halal income (e.g., from interest) before the Zakat calculation.
*   **Professional Income**: We support Dr. Al-Qaradawi's view on *Zakat al-Mustafad*, allowing you to calculate Zakat either on your Gross income (like a harvest) or Net income (after basic needs).
*   **The "Dam' al-Amwal" Principle**: To ensure the computation is most beneficial for the poor (*Anfa' lil-fuqara*), we follow the **Hanafi** methodology of aggregating all your monetary assets (Gold, Silver, Cash, Stocks) into a single pot to check against the Nisab, rather than isolating them.
