use zakat::prelude::*;
use zakat::types::{ZakatDetails, ZakatError};
use zakat::traits::CalculateZakat;
use zakat::config::ZakatConfig;
use rust_decimal_macros::dec;

struct FailingAsset;

impl CalculateZakat for FailingAsset {
    fn calculate_zakat(&self, _config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        Err(ZakatError::CalculationError("Intentional Failure".to_string(), None))
    }
}

#[test]
fn test_portfolio_partial_failure() {
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_silver_price(1);
    
    let valid_asset = PreciousMetals::new()
        .weight(100)
        .metal_type(WealthType::Gold);
        
    let failing_asset = FailingAsset;
    
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
        assert!(error.to_string().contains("Intentional Failure"));
    } else {
        panic!("Expected failure variant");
    }
    
    // Check totals (should include valid assets)
    // 100g Gold * $100 = $10,000
    assert_eq!(report.total_assets, dec!(10000.0));
}
