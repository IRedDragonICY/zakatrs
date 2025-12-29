use rust_decimal_macros::dec;
use zakat::prelude::*;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Live Ticker Simulation ===");

    // 1. Initialize Assets ONCE (e.g. at app startup or user input)
    // Notice we do NOT pass config here anymore.
    let gold_stash = PreciousMetals::new(100, WealthType::Gold)?.with_label("Grandma's Necklace");
    let trade_goods_assets = BusinessAssets::builder()
        .cash(10_000)
        .inventory(5_000)
        .liabilities(2_000)
        .build()
        .expect("Assets valid");
    let trade_goods = BusinessZakatCalculator::new(trade_goods_assets)
        .with_label("Corner Store");

    // 2. Create a Portfolio holding these assets
    let portfolio = ZakatPortfolio::new()
        .add(gold_stash)
        .add(trade_goods);

    // 3. Simulation Day 1: Gold is Cheap ($40/g)
    // Nisab = 85 * 40 = $3,400.
    let config_day1 = ZakatConfig::new(40, 1)?;
    
    println!("\n--- Day 1 (Gold ${}/g) ---", config_day1.gold_price_per_gram);
    let result_day1 = portfolio.calculate_total(&config_day1);
    if !result_day1.errors.is_empty() {
        eprintln!("Errors: {:?}", result_day1.errors);
        return Err("Calculation failed".into());
    }
    print_summary("Day 1", &result_day1);

    // 4. Simulation Day 2: Gold Spikes ($80/g)
    // Nisab = 85 * 80 = $6,800.
    // We do NOT need to recreate assets or portfolio. Just pass new config.
    let config_day2 = ZakatConfig::new(80, 1)?;

    println!("\n--- Day 2 (Gold ${}/g) ---", config_day2.gold_price_per_gram);
    let result_day2 = portfolio.calculate_total(&config_day2);
    if !result_day2.errors.is_empty() {
         eprintln!("Errors: {:?}", result_day2.errors);
         return Err("Calculation failed".into());
    }
    print_summary("Day 2", &result_day2);

    // Verification
    assert_ne!(result_day1.total_zakat_due, result_day2.total_zakat_due);
    println!("\n[SUCCESS] Dynamic pricing updated Zakat calculation without re-initialization.");

    Ok(())
}

fn print_summary(day: &str, result: &PortfolioReport) {
    println!("{} Summary:", day);
    println!("Total Assets : ${}", result.total_assets);
    // Portfolio doesn't have a single Nisab threshold, it's per asset type.
    println!("Zakat Due    : ${}", result.total_zakat_due);
    if result.total_zakat_due > dec!(0) {
        println!("Status       : PAYABLE");
    } else {
        println!("Status       : NOT PAYABLE (Below Nisab)");
    }
}
