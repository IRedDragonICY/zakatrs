use zakat::prelude::*;
use zakat::pricing::{PriceProvider, Prices};
use zakat::types::ZakatError;
use rust_decimal::Decimal;

use std::time::Duration;
use tokio::time::sleep;
use async_trait::async_trait;

/// Mock provider simulating an external API call
struct MockApiPriceProvider {
    gold: rust_decimal::Decimal,
    silver: rust_decimal::Decimal,
}

#[async_trait]
impl PriceProvider for MockApiPriceProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // Simulate network latency
        println!("Connecting towards 'live' API...");
        sleep(Duration::from_millis(500)).await;
        println!("Fetched prices successfully.");
        
        Ok(Prices {
            gold_per_gram: self.gold,
            silver_per_gram: self.silver,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Async Zakat Calculation with Live Pricing ---");

    // 1. Setup Live Price Provider
    let provider = MockApiPriceProvider {
        gold: Decimal::from(95),    // Assume fetched from API
        silver: Decimal::from_f64_retain(1.20).unwrap(),
    };

    // 2. Async Config Initialization
    // We can also use config.refresh_prices(&provider) if we had an existing config.
    let config = ZakatConfig::from_provider(&provider).await?;
    println!("Config initialized with Gold: {}, Silver: {}", config.gold_price_per_gram, config.silver_price_per_gram);

    // 3. Create Async Portfolio
    // Note: existing calculators (PreciousMetals, BusinessZakat) are used directly!
    // This is because of the blanket impl: impl AsyncCalculateZakat for T where T: CalculateZakat
    let portfolio = AsyncZakatPortfolio::new()
        .add(PreciousMetals::new()
            .weight(100)
            .metal_type(WealthType::Gold)) // 100g Gold
        .add(BusinessZakat::new()
            .cash(5000)
            .inventory(2000)
            .liabilities(1000));

    // 4. Calculate Asynchronously
    println!("\nCalculating Portfolio...");
    let result = portfolio.calculate_total_async(&config).await;

    println!("Total Assets: {}", result.total_assets);
    println!("Total Zakat Due: {}", result.total_zakat_due);
    
    for detail in result.successes() {
        println!(" - {:?}: {} (Payable: {})", detail.wealth_type, detail.zakat_due, detail.is_payable);
        if let Some(reason) = &detail.status_reason {
            println!("   Reason: {}", reason);
        }
    }

    if !result.failures().is_empty() {
        println!("\nFailures:");
        for fail in result.failures() {
            println!("{:?}", fail);
        }
    }
    
    Ok(())
}
