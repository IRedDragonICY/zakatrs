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
- Portfolio aggregation (Dam' al-Amwal)
- **Asset Labeling** (e.g., "Main Store", "Crypto Wallet")
- **Input Sanitization & Validation** (Rejects negative values, ensures safe configuration)
- **Flexible Configuration** (Env Vars, JSON, Fluent Builder)
- **Fiqh Compliance** (Jewelry exemptions, Madhab-specific rules, Hawl requirements)
- **Async Support** (Optional integration with `tokio` and `async-trait`)
- **Live Pricing Interface** (e.g. for API integration)
- **Detailed Reporting** (Livestock in-kind details, calculation traces, metadata support)

## Install

With Async Support (Default):
```toml
[dependencies]
zakat = "0.4.1"
rust_decimal = "1.39"
tokio = { version = "1", features = ["full"] } # Required if using async features
```

Synchronous Only (Lighter weight):
```toml
[dependencies]
zakat = { version = "0.4.1", default-features = false }
rust_decimal = "1.39"
```

## Usage

### Business Zakat

> **Note:** You can pass standard Rust types (`i32`, `f64`, `&str`) directly to all constructors for ease of use.

```rust
use zakat::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ZakatConfig::new(65, 1)?; // gold $65/g, silver $1/g

    // Builder pattern with validation
    // Validates inputs (non-negative) and configuration on .build()
    let store = BusinessZakatBuilder::default()
        .cash_on_hand(10_000)
        .inventory_value(50_000)
        .label("Main Store")
        .hawl_satisfied(true)
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
let assets = BusinessZakatBuilder::default()
    .cash_on_hand(50000)
    .inventory_value(20000)
    .receivables(5000)
    .short_term_liabilities(1000)
    .liabilities_due_now(500) // Deductible immediate debt
    .label("Tech Startup")
    .hawl_satisfied(true)
    .build()?;
```

### Portfolio Management

Handles multiple assets with "Dam' al-Amwal" (Wealth Aggregation) logic.

```rust
use zakat::prelude::*;
use zakat::portfolio::PortfolioStatus;

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

    let result = portfolio.calculate_total(&config);
    println!("Total Zakat Due: ${}", result.total_zakat_due);
    
    // robust error handling for partial failures
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
            .add(BusinessZakatBuilder::default()
                .cash_on_hand(10_000)
                .build()?);
                
        let result = portfolio.calculate_total_async(&config).await;
        println!("Total Due: {}", result.total_zakat_due);
    }
    Ok(())
}
```

### Configuration

Refactored to be flexible and safe.

```rust
use zakat::prelude::*;

// Load from Environment Variables (ZAKAT_GOLD_PRICE, etc.)
let config = ZakatConfig::from_env()?;

// Or load from JSON
let config = ZakatConfig::try_from_json("config.json")?;

// Or using Fluent Builder (with Validation)
let config = ZakatConfigBuilder::default()
    .gold_price(100.0)
    .silver_price(1.0)
    .madhab(Madhab::Hanafi)
     // Validates that adequate prices are set for the chosen Madhab/Standard
    .build()?;
```

### Advanced Assets (Jewelry & Livestock)

```rust
use zakat::prelude::*;

// Personal Jewelry (Exempt in Shafi/Maliki, Payable in Hanafi)
let necklace = PreciousMetals::new(100.0, WealthType::Gold)?
    .with_usage(JewelryUsage::PersonalUse)
    .with_label("Wife's Wedding Necklace");

// Livestock Reporting
let prices = LivestockPricesBuilder::default()
    .sheep_price(200)
    .cow_price(1500)
    .camel_price(3000)
    .build()?;
    
let camels = LivestockAssets::new(30, LivestockType::Camel, prices);

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
