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
    let config = ZakatConfig::new(100, 1).unwrap();
    
    let valid_asset = PreciousMetals::new(100, WealthType::Gold).unwrap();
    let failing_asset = FailingAsset;
    
    let portfolio = ZakatPortfolio::new()
        .add(valid_asset)
        .add(failing_asset);
        
    let report = portfolio.calculate_total(&config);
    
    // Check successful results
    assert_eq!(report.details.len(), 1);
    assert_eq!(report.details[0].wealth_type, WealthType::Gold);
    assert!(report.details[0].is_payable);
    
    // Check errors
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].contains("Intentional Failure"));
    
    // Check totals (should include valid assets)
    // 100g Gold * $100 = $10,000
    assert_eq!(report.total_assets, dec!(10000.0));
}
