use rust_decimal_macros::dec;
use zakat::{ZakatConfig, ZakatPortfolio, WealthType};
use zakat::maal::precious_metals::{PreciousMetal};
use zakat::maal::investments::{InvestmentAssets, InvestmentType};
use zakat::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};

fn main() {
    println!("=== Mr. Ahmad Zakat Scenario ===");

    // Scenario:
    // - Income: $5,000/month (Net/Gross base).
    // - Gold: 100g.
    // - Crypto: $20,000.
    // - Personal Debt: $2,000.
    // - Gold Price: $50/gram.
    
    // Config - NEW ERGONOMIC API: No dec!() needed!
    let config = ZakatConfig::new(50, 1); // Gold $50/g, Silver $1/g
    println!("Configuration: Gold Price = ${}/g", config.gold_price_per_gram);
    println!("Nisab Threshold (Gold): ${}", config.gold_price_per_gram * config.get_nisab_gold_grams());

    // 1. Income - integers work directly!
    let income_calc = IncomeZakatCalculator::new(
        5000, 
        0, 
        IncomeCalculationMethod::Gross, 
        &config
    ).unwrap();
    
    // 2. Gold - integers work directly!
    let gold_calc = PreciousMetal::new(
        100, 
        WealthType::Gold, 
        &config
    ).unwrap();
    
    // 3. Crypto - integers work directly!
    let crypto_calc = InvestmentAssets::new(
        20000, 
        InvestmentType::Crypto, 
        &config
    ).unwrap();
    
    // 4. Portfolio with Debt Deduction on Crypto
    let portfolio = ZakatPortfolio::new()
        .add_calculator(income_calc) // $5000 * 2.5% = $125
        .add_calculator(gold_calc)   // $5000 * 2.5% = $125 (100g * 50)
        .add_calculator(crypto_calc.with_debt(dec!(2000.0))); // ($20,000 - $2,000) * 2.5% = $450
        
    let result = portfolio.calculate_total(&config).unwrap();
    
    println!("\n--- Portfolio Result ---");
    println!("Total Assets: ${}", result.total_assets);
    println!("Total Zakat Due: ${}", result.total_zakat_due);
    
    println!("\n--- Breakdown ---");
    for detail in &result.details {
        println!("Type: {:?}, Net Assets: ${}, Zakat: ${}", detail.wealth_type, detail.net_assets, detail.zakat_due);
    }

    // Assertions to ensure correctness (Self-verifying example)
    // Total: 125 + 125 + 450 = 700.0
    assert_eq!(result.total_zakat_due, dec!(700.0));
    println!("\n[SUCCESS] Calculation verified successfully.");
}
