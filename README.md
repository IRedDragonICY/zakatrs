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
╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   
```

# Zakat

[![Crates.io](https://img.shields.io/crates/v/zakat.svg)](https://crates.io/crates/zakat)
[![Docs.rs](https://docs.rs/zakat/badge.svg)](https://docs.rs/zakat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust library for Islamic Zakat calculation. Uses `rust_decimal` for precision.

## Features

- Gold, Silver, Business, Agriculture, Livestock, Mining & Rikaz
- Stocks, Mutual Funds, Crypto (as liquid assets)
- Professional Income (Gross/Net)
- Zakat Fitrah
- Configurable Nisab thresholds
- Portfolio aggregation
- **Asset Labeling** (e.g., "Main Store", "Crypto Wallet")
- **Input Sanitization** (Rejects negative values)
- **Flexible Configuration** (Env Vars, JSON, Fluent Builder)
- **Fiqh Compliance** (Jewelry exemptions, Madhab-specific rules)
- **Detailed Reporting** (Livestock in-kind details, metadata support)

## Install

```toml
[dependencies]
zakat = "0.4.0"
rust_decimal = "1.39"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Usage

### Business Zakat

> **Note:** You can pass standard Rust types (`i32`, `f64`, `&str`) directly to all constructors. There is no need to manually convert to `Decimal` or use the `dec!()` macro anymore.

```rust
use zakat::{ZakatConfig, CalculateZakat};
use zakat::maal::business::BusinessZakat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZakatConfig::new(65, 1)?; // gold $65/g, silver $1/g

    // 1. Simple Case: Trading Goods (Cash + Inventory)
    // Now unified into a single builder step!
    let store = BusinessZakat::builder()
        .cash(10_000)
        .inventory(50_000)
        .label("Main Store")
        .hawl(true)
        .build()?;

    // Calculate directly
    let result = store.calculate_zakat(&config)?;

    if result.is_payable {
        println!("Zakat for {}: ${}", result.label.unwrap_or_default(), result.zakat_due);
    }
    Ok(())
}
```

### Advanced Usage (Builder Pattern)

For complex scenarios involving debts and receivables:

```rust
let assets = BusinessZakat::builder()
    .cash(50000)
    .inventory(20000)
    .receivables(5000)
    .liabilities(1000)
    .debt(500) // Deductible immediate debt
    .build()?;
```

### Portfolio

```rust
use zakat::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZakatConfig::new(65, 1)?;

    let portfolio = ZakatPortfolio::new()
        .add(IncomeZakatCalculator::new(
            5000, 0, IncomeCalculationMethod::Gross
        )?.with_label("Monthly Salary"))
        .add(PreciousMetals::new(
            100, WealthType::Gold
        )?.with_label("Wife's Gold"))
        .add(InvestmentAssets::new(
            20000, InvestmentType::Crypto
        )?.with_debt(2000)?.with_label("Binance Portfolio"));

    let result = portfolio.calculate_total(&config)?;
    println!("Total Zakat Due: ${}", result.total_zakat_due);
    
    // Check for partial failures
    if !result.is_clean() {
        println!("Warning: {} items failed to calculate", result.failures().len());
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

### Configuration

```rust
use zakat::{ZakatConfig, Madhab, AssetBuilder};

// Load from Environment Variables (ZAKAT_GOLD_PRICE, etc.)
let config = ZakatConfig::from_env()?;

// Or load from JSON
let config = ZakatConfig::try_from_json("config.json")?;

// Or using Value Builder
let config = ZakatConfig::builder()
    .gold_price(100.0)
    .silver_price(1.0)
    .madhab(Madhab::Hanafi)
    .build()?;
```

### Advanced Assets (Jewelry & Livestock)

```rust
use zakat::maal::precious_metals::{PreciousMetal, JewelryUsage};
use zakat::maal::livestock::{LivestockAssets, LivestockType, LivestockPrices};

// Personal Jewelry (Exempt in Shafi/Maliki, Payable in Hanafi)
let necklace = PreciousMetals::new(100.0, WealthType::Gold)?
    .with_usage(JewelryUsage::PersonalUse)
    .with_label("Wife's Wedding Necklace");

// Livestock Reporting
let prices = LivestockPrices::builder()
    .sheep_price(200)
    .cow_price(1500)
    .camel_price(3000)
    .build()?;
let camels = LivestockAssets::new(30, LivestockType::Camel, prices);

let result = camels.calculate_zakat(&config)?;

if result.is_payable {
    // Access detailed "in-kind" payment info
    use zakat::types::PaymentPayload;
    if let PaymentPayload::Livestock { description, .. } = result.payload {
        println!("Pay Due: {}", description);
        // Output: "Pay Due: 1 Bint Makhad"
    }
}
```

### Custom Nisab

```rust
use zakat::ZakatConfig;

let config = ZakatConfig::new(65.0, 1.0)?
    .with_gold_nisab(87)
    .with_agriculture_nisab(700);
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

## Contributing

1. Add tests
2. Use `rust_decimal`
3. Run `cargo test`

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
