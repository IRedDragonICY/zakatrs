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
```

# Zakat

[![Crates.io](https://img.shields.io/crates/v/zakat.svg)](https://crates.io/crates/zakat)
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

### Usage
```rust
use zakat::i18n::ZakatLocale;

let details = calculator.calculate_zakat(&config)?;

// Get localized summary
println!("{}", details.summary_in(ZakatLocale::IdID));
// Output: "Aset: WAJIB ZAKAT - Jumlah Wajib: Rp25.000,00"

// Get detailed localized explanation
println!("{}", details.explain_in(ZakatLocale::ArSA));
```

## 📦 Installation

With Async Support (Default):
```toml
[dependencies]
zakat = "0.17.0"
rust_decimal = "1.39"
tokio = { version = "1", features = ["full"] } # Required if using async features
```

Synchronous Only (Lighter weight):
```toml
[dependencies]
zakat = { version = "0.17.0", default-features = false }
rust_decimal = "1.39"
```

## Usage

### Business Zakat

> **Note:** v0.7 continues to use the **Fluent API** introduced in v0.5. No more Builders!

```rust
use zakat::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZakatConfig::new()
        .with_gold_price(65)  // $65/g
        .with_silver_price(1); // $1/g

    // Fluent API: Infallible construction, chained setters
    // Newer "Semantic Constructor" (v0.7+) makes common cases even faster:
    let store = BusinessZakat::cash_only(10_000)
        .inventory(50_000)
        .label("Main Store");
        // .hawl(true) is default in cash_only(), so we can skip it if strictly satisfied

    // Validation & Calculation happens here
    // Flexible arguments: pass &config, Option<&config>, or () for defaults
    let result = store.calculate_zakat(&config)?; 
    // Or simply: store.calculate()?; 

    if result.is_payable {
        println!("Zakat for {}: ${}", result.label.unwrap_or_default(), result.zakat_due);
    }
    
    // NEW: Get a human-readable trace of the calculation
    println!("{}", result.explain());
    
    Ok(())
}
```

### Advanced Usage (Complex Scenarios)

For complex scenarios involving debts and receivables:

```rust
let assets = BusinessZakat::new()
    .cash(50000)
    .inventory(20000)
    .receivables(5000)
    .liabilities(1000)
    .debt(500) // Deductible immediate debt
    .label("Tech Startup")
    .hawl(true);
```

### Portfolio Management

Handles multiple assets with "Dam' al-Amwal" (Wealth Aggregation) logic.

```rust
use zakat::prelude::*;
use zakat::portfolio::PortfolioStatus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZakatConfig::new()
        .with_gold_price(65)
        .with_silver_price(1);

    let portfolio = ZakatPortfolio::new()
        .add(IncomeZakatCalculator::from_salary(5000)
            .label("Monthly Salary"))
        .add(PreciousMetals::gold(100)
            .label("Wife's Gold"))
        .add(InvestmentAssets::crypto(20000)
            .debt(2000)
            .label("Binance Portfolio"));

    let result = portfolio.calculate_total(&config);
    println!("Total Zakat Due: ${}", result.total_zakat_due);
    
    // Robust error handling for partial failures
    match result.status {
        PortfolioStatus::Complete => println!("All assets calculated successfully."),
        PortfolioStatus::Partial => {
            println!("Warning: Some assets failed calculation.");
            for failure in result.failures() {
                 println!("Failed item: {:?}", failure);
            }
        }
        PortfolioStatus::Failed => println!("Critical: All asset calculations failed."),
    }

    // Iterate successful details
    for detail in result.successes() {
        if let Some(label) = &detail.label {
            println!(" - {}: ${}", label, detail.zakat_due);
        }
    }
    Ok(())
}
```

### Dynamic Portfolio Operations

New in v0.6: Manage assets dynamically using stable UUIDs.

```rust
use zakat::prelude::*;

fn main() {
    let mut portfolio = ZakatPortfolio::new();
    
    // Add returns the ID
    let (portfolio, id_1) = portfolio.add_with_id(
        BusinessZakat::new().cash(10_000).label("Branch A")
    );
    
    // Or push to mutable reference
    let mut portfolio = portfolio;
    let id_2 = portfolio.push(
        BusinessZakat::cash_only(5_000).label("Branch B")
    );
    
    // Replace an asset (e.g. updating values)
    portfolio.replace(id_1, BusinessZakat::new().cash(12_000).label("Branch A Updated")).unwrap();
    
    // Remove an asset
    portfolio.remove(id_2);
}
```

### Async & Live Pricing (Optional)

Enable the `async` feature to use these capabilities.

```rust
use zakat::prelude::*;
use zakat::pricing::{PriceProvider, Prices};

struct MockApi;

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl PriceProvider for MockApi {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // Simulate API call
        Ok(Prices::new(90.0, 1.2)?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "async")]
    {
        let api = MockApi;
        // Initialize config from provider
        let config = ZakatConfig::from_provider(&api).await?;
        
        let portfolio = AsyncZakatPortfolio::new()
            .add(BusinessZakat::new()
                .cash(10_000));
                
        let result = portfolio.calculate_total_async(&config).await;
        println!("Total Due: {}", result.total_zakat_due);
    }
    Ok(())
}
```

### Configuration

Flexible and safe configuration options.

```rust
use zakat::prelude::*;

// Load from Environment Variables (ZAKAT_GOLD_PRICE, etc.)
let config = ZakatConfig::from_env()?;

// Or load from JSON
let config = ZakatConfig::try_from_json("config.json")?;

// Or using Fluent API
let config = ZakatConfig::new()
    .with_gold_price(100)
    .with_silver_price(1)
    .with_madhab(Madhab::Hanafi);

// NEW (v0.7+): Quick Config Presets
let config = ZakatConfig::hanafi(100, 1); // Sets Madhab=Hanafi, Nisab=LowerOfTwo, Prices
let config = ZakatConfig::shafi(100);     // Sets Madhab=Shafi, Nisab=Gold, Prices

// NEW (v0.14+): Merge Configurations (Layering)
let base_config = ZakatConfig::default();
let env_config = ZakatConfig::from_env().unwrap_or_default();
// Env values override defaults
let final_config = base_config.merge(env_config);
```

### Custom Zakat Strategy (Advanced)

Create custom calculation rules beyond the standard Madhabs:

```rust
use zakat::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
struct GregorianTaxStrategy;

impl ZakatStrategy for GregorianTaxStrategy {
    fn get_rules(&self) -> ZakatRules {
        // v0.7 allows fluent configuration with direct literals
        ZakatRules::default()
            .with_nisab_standard(NisabStandard::Gold)
            .with_trade_goods_rate(0.02577) // 2.577%
    }
    }
}

fn main() {
    // Use custom strategy with with_madhab() (accepts any impl ZakatStrategy)
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_madhab(GregorianTaxStrategy);
    
    // Or share strategy across configs with Arc
    let shared = Arc::new(GregorianTaxStrategy);
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_strategy(shared);
}

### Advanced Assets (Jewelry & Livestock)

```rust
use zakat::prelude::*;

// Personal Jewelry (Exempt in Shafi/Maliki, Payable in Hanafi)
let necklace = PreciousMetals::gold(100)
    .usage(JewelryUsage::PersonalUse)
    .label("Wife's Wedding Necklace");

// Livestock Reporting
let prices = LivestockPrices::new()
    .sheep_price(200)
    .cow_price(1500)
    .camel_price(3000);
    
let camels = LivestockAssets::new()
    .count(30)
    .animal_type(LivestockType::Camel)
    .prices(prices);

let result = camels.calculate_zakat(&config)?;

if result.is_payable {
    // Access detailed "in-kind" payment info
    if let crate::types::PaymentPayload::Livestock { description, .. } = result.payload {
        println!("Pay Due: {}", description);
        // Output: "Pay Due: 1 Bint Makhad"
    }
}
```

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

## Migrating from v0.4 (Builder Pattern)

v0.5 removes all `*Builder` structs. Migration is straightforward:

```diff
- let assets = BusinessZakatBuilder::default()
-     .cash_on_hand(10_000)
-     .build()?;
+ let assets = BusinessZakat::new()
+     .cash(10_000);
```

Key changes:
- `::builder()` → `::new()` (now infallible, returns `Self`)
- `.build()?` → removed, object is ready immediately
- Validation errors now occur at `.calculate_zakat()` instead of `.build()`
- Errors include Asset Labels for better debugging

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

> *"Those who spend their wealth in the cause of Allah..."* — **Al-Baqarah 2:262**

## License

MIT

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
*   **Stocks & Crypto**: We treat these as Trade Goods (*Urud*), subjecting them to a 2.5% rate on their market value, consistent with **[AAOIFI Standard 35](https://aaoifi.com)**.
*   **Professional Income**: We support Dr. Al-Qaradawi's view on *Zakat al-Mustafad*, allowing you to calculate Zakat either on your Gross income (like a harvest) or Net income (after basic needs).
*   **The "Dam' al-Amwal" Principle**: To ensure the computation is most beneficial for the poor (*Anfa' lil-fuqara*), we follow the **Hanafi** methodology of aggregating all your monetary assets (Gold, Silver, Cash, Stocks) into a single pot to check against the Nisab, rather than isolating them.
