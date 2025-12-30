use zakat::prelude::*;
// use zakat::types::{ZakatDetails, ZakatError};
// use zakat::traits::CalculateZakat;
use zakat::config::ZakatConfig;
use rust_decimal_macros::dec;
// use uuid::Uuid;

#[test]
fn test_portfolio_partial_failure() {
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_silver_price(1);
    
    // Valid Asset: Gold
    let valid_asset = PreciousMetals::new()
        .weight(100)
        .metal_type(WealthType::Gold);
        
    // Failing Asset: PreciousMetals without metal type set
    // This will fail validation in calculate_zakat
    let failing_asset = PreciousMetals::new()
        .weight(50); // Missing metal_type
    
    let portfolio = ZakatPortfolio::new()
        .add(valid_asset)
        .add(failing_asset);
        
    let report = portfolio.calculate_total(&config);
    
    // Check successful results
    let successes = report.successes();
    assert_eq!(successes.len(), 1);
    assert_eq!(successes[0].wealth_type, WealthType::Gold);
    assert!(successes[0].is_payable);
    
    // Check errors
    let failures = report.failures();
    assert_eq!(failures.len(), 1);
    
    if let PortfolioItemResult::Failure { error, .. } = failures[0] {
        // Expect "Metal type must be specified" error
        assert!(error.to_string().contains("Metal type must be specified"));
    } else {
        panic!("Expected failure variant");
    }
    
    // Check totals (should include valid assets)
    // 100g Gold * $100 = $10,000
    assert_eq!(report.total_assets, dec!(10000.0));
}
