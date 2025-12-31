use rust_decimal::Decimal;
use zakat::{ZakatConfig, ZakatPortfolio, WealthType};
use zakat::maal::precious_metals::{PreciousMetals};
use zakat::maal::investments::{InvestmentAssets, InvestmentType};
use zakat::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mr. Ahmad Zakat Scenario ===");

    // Scenario:
    // - Income: $5,000/month (Net/Gross base).
    // - Gold: 100g.
    // - Crypto: $20,000.
    // - Personal Debt: $2,000.
    // - Gold Price: $50/gram.
    
    // Config - NEW ERGONOMIC API
    let config = ZakatConfig::new()
        .with_gold_price(50)
        .with_silver_price(1);
    
    println!("Configuration: Gold Price = ${}/g", config.gold_price_per_gram);
    println!("Nisab Threshold (Gold): ${}", config.gold_price_per_gram * config.get_nisab_gold_grams());

    // 1. Income - integers work directly!
    let income_calc = IncomeZakatCalculator::new()
        .income(5000)
        .expenses(0)
        .method(IncomeCalculationMethod::Gross)
        .hawl(true)
        .label("Monthly Salary");
    
    // 2. Gold - integers work directly!
    let gold_calc = PreciousMetals::new()
        .weight(100)
        .metal_type(WealthType::Gold)
        .label("Wife's Gold Stash");
    
    // 3. Crypto - integers work directly!
    let crypto_calc = InvestmentAssets::new()
        .value(20000)
        .kind(InvestmentType::Crypto)
        .hawl(true)
        .label("Bitcoin Holding");
    
    // 4. Portfolio with Debt Deduction on Crypto
    let portfolio = ZakatPortfolio::new()
        .add(income_calc) // $5000 * 2.5% = $125
        .add(gold_calc)   // $5000 * 2.5% = $125 (100g * 50)
        .add(crypto_calc.debt(2000.0)); // ($20,000 - $2,000) * 2.5% = $450
        
    let result = portfolio.calculate_total(&config);
    
    if !result.is_clean() {
        println!("Likely Errors: {:?}", result.failures());
    }
    
    println!("\n--- Portfolio Result ---");
    println!("Total Assets: ${}", result.total_assets);
    println!("Total Zakat Due: ${}", result.total_zakat_due);
    
    println!("\n--- Breakdown ---");
    for detail in result.successes() {
        print!("Asset: {:<20} | Type: {:<12}", detail.label.as_deref().unwrap_or("Unknown"), format!("{:?}", detail.wealth_type));
        println!(" | Net: ${:<10} | Zakat: ${}", detail.net_assets, detail.zakat_due);
    }

    // Assertions to ensure correctness (Self-verifying example)
    // Total: 125 + 125 + 450 = 700.0
    assert_eq!(result.total_zakat_due, Decimal::from(700));
    println!("\n[SUCCESS] Calculation verified successfully.");
    Ok(())
}
