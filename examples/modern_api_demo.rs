
use zakat::prelude::*;
use zakat::config::ZakatConfig;
use zakat::maal::business::BusinessZakat;
use zakat::maal::precious_metals::PreciousMetals;
use zakat::portfolio::ZakatPortfolio;


fn main() {
    println!("=== Zakat Modern API Demo ===\n");

    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // 1. Semantic Configuration (Hanafi Preset)
    // Gold Price: $80/g, Silver Price: $1/g
    let config = ZakatConfig::hanafi(80, 1);
    println!("Configuration Loaded: {:?}", config.cash_nisab_standard);
    println!("Nisab Threshold: ${}\n", config.get_monetary_nisab_threshold());

    // 2. Fluent Portfolio Construction
    // Using new Semantic Constructors
    let portfolio = ZakatPortfolio::new()
        .add(BusinessZakat::cash_only(10_000).label("Main Cash Register"))
        .add(PreciousMetals::gold(100).label("Gold Bars")) // 100g Gold
        .add(PreciousMetals::silver(50).label("Silver Coins")); // 50g Silver

    // 3. Calculation
    let result = portfolio.calculate_total(&config);

    if result.status == zakat::portfolio::PortfolioStatus::Complete {
        println!("Portfolio Calculation Successful!");
        println!("Total Assets: ${}", result.total_assets);
        println!("Total Zakat Due: ${}\n", result.total_zakat_due);

        println!("--- Asset Breakdown ---");
        for details in result.successes() {
            println!("{}", details.summary(&zakat::i18n::default_translator()));
        }
    } else {
        println!("Calculation Failed!");
    }

    println!("\n=== Demonstration of Unified Error Diagnostics ===");
    
    // 4. Force an Error (Negative Input)
    let bad_asset = BusinessZakat::cash_only(-500).label("Negative Cash");
    
    // We calculate directly to get the error
    let error_result = bad_asset.calculate_zakat(&config);

    if let Err(e) = error_result {
        println!("{}", e.report(&zakat::i18n::default_translator()));
    }
}
