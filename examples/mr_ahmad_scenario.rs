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
    
    // Config
    let config = ZakatConfig::new(dec!(50.0), dec!(1.0)); // Gold $50
    println!("Configuration: Gold Price = ${}/g", config.gold_price_per_gram);
    println!("Nisab Threshold (Gold): ${}", config.gold_price_per_gram * config.get_nisab_gold_grams());

    // 1. Income
    let income_calc = IncomeZakatCalculator::new(
        dec!(5000.0), 
        dec!(0.0), 
        IncomeCalculationMethod::Gross, 
        &config
    ).unwrap();
    
    // 2. Gold
    let gold_calc = PreciousMetal::new(
        dec!(100.0), 
        WealthType::Gold, 
        &config
    ).unwrap();
    
    // 3. Crypto
    let crypto_calc = InvestmentAssets::new(
        dec!(20000.0), 
        InvestmentType::Crypto, 
        &config
    ).unwrap();
    
    // 4. Portfolio with Debt Deduction on Crypto
    let portfolio = ZakatPortfolio::new()
        .add_calculator(income_calc) // $5000 * 2.5% = $125
        .add_calculator(gold_calc)   // $5000 * 2.5% = $125 (100g * 50)
        .add_calculator_with_debt(crypto_calc, dec!(2000.0)); // ($20,000 - $2,000) * 2.5% = $450
        
    let result = portfolio.calculate_total().unwrap();
    
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
