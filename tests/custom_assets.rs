use rust_decimal_macros::dec;
use zakat::assets::CustomAsset;
use zakat::portfolio::ZakatPortfolio;
use zakat::config::ZakatConfig;
use zakat::traits::CalculateZakat;

#[test]
fn test_custom_asset_calculation() {
    let asset = CustomAsset::new(
        "Real Estate Fund", 
        dec!(100_000), 
        dec!(0.025), 
        dec!(85) * dec!(65) // Approx Gold Nisab Value
    );

    let config = ZakatConfig::default();
    let details = asset.calculate_zakat(&config).expect("Calculation failed");

    assert!(details.is_payable);
    assert_eq!(details.zakat_due, dec!(2500));
    assert_eq!(details.wealth_type, zakat::types::WealthType::Other("Custom".to_string()));
    assert_eq!(details.label, Some("Real Estate Fund".to_string()));
}

#[test]
fn test_custom_asset_below_nisab() {
    let asset = CustomAsset::new(
        "Small Savings", 
        dec!(100), 
        dec!(0.025), 
        dec!(5000) 
    );

    let config = ZakatConfig::default();
    let details = asset.calculate_zakat(&config).expect("Calculation failed");

    assert!(!details.is_payable);
    assert_eq!(details.zakat_due, dec!(0));
}

#[test]
fn test_custom_asset_in_portfolio() {
    let mut portfolio = ZakatPortfolio::new();
    
    portfolio = portfolio.add(CustomAsset::new(
        "Crypto Staking", 
        dec!(50_000), 
        dec!(0.025), 
        dec!(1000)
    ));

    // Add standard asset too
    portfolio = portfolio.add_business(|b| b
        .cash(dec!(10000))
        .liabilities(dec!(0))
    );

    let config = ZakatConfig {
        gold_price_per_gram: dec!(100.0),
        silver_price_per_gram: dec!(1.0),
        ..ZakatConfig::default()
    };
    let result = portfolio.calculate_total(&config);

    assert!(result.is_clean());
    // Custom: 50k * 2.5% = 1250
    // Business: 10k * 2.5% = 250 (assuming gold price default allows it, or nisab is low enough)
    // Total should include Custom.
    
    let custom_res = result.results.iter().find(|r| {
        if let zakat::portfolio::PortfolioItemResult::Success { details, .. } = r {
             details.wealth_type == zakat::types::WealthType::Other("Custom".to_string())
        } else {
            false
        }
    });

    assert!(custom_res.is_some());
}
