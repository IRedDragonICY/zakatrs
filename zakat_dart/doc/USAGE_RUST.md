# Rust Usage Guide 🦀

Configuration and advanced usage examples for the `zakat` crate.

## Basic Business Zakat

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

## Advanced Business Scenarios

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

## Portfolio Management

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

## Dynamic Portfolio Operations

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

## Async & Live Pricing

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

## Configuration

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

## Custom Zakat Strategy

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
```

## Advanced Assets (Jewelry & Livestock)

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

## Hawl & Purification

New in v0.19.0: Strict Fiqh logic for lunar years and income purification.

### Hawl (Lunar Year) Tracking
Zakat is only due if an asset is held for 1 Lunar Year (approx 354 days). You can now track this precisely:

```rust
use chrono::NaiveDate;

let acquired_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

let store = BusinessZakat::new()
    .cash(10_000.0)
    .acquired_on(acquired_date); // Sets exact start date

// If calculated today (e.g. 2023-06-01), specific logic checks if 354 days passed.
// .acquired_on() overrides manual .hawl(true)
```

### Stock Purification (Tathir)
For investments in mixed companies (halal business but some interest income), you must purify the non-halal portion *before* Zakat.

```rust
let stock = InvestmentAssets::new()
    .value(10_000.0)
    .kind(InvestmentType::Stock)
    .purify(0.05); // 5% of value is impure

// Result:
// Gross Value: 10,000
// Impure (Deducted): 500
// Net Zakatable: 9,500
// Zakat Due (2.5%): 237.50
```
This purification step is clearly visible in `result.explain()`.
