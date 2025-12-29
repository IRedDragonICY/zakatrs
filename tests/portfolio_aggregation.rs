use zakat::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn test_portfolio_aggregation_mix_gold_and_cash() {
    // Scenario:
    // Gold Price: $100/g. Nisab (85g) = $8,500.
    // User has:
    // 1. 50g Gold = $5,000. (Below $8,500 independently)
    // 2. Cash = $4,000. (Below $8,500 independently)
    //
    // Total Monetary Wealth = $9,000.
    // This EXCEEDS $8,500.
    // Zakat should be due on the total $9,000.
    // Zakat Due = $9,000 * 0.025 = $225.

    let config = ZakatConfig::new(
        100.0, // Gold price
        1.0    // Silver price (irrelevant here if using Gold standard or if Gold is higher)
    ).unwrap().with_madhab(Madhab::Shafi); // Explicitly Gold standard for simplicity

    let gold_asset = PreciousMetals::new(
        50.0, 
        WealthType::Gold
    ).expect("Valid gold input");

    // Using BusinessAssets to represent Cash roughly, or closest equivalent if Cash isn't explicit
    // Actually, BusinessAssets::new(cash, inventory, ...)
    let cash_assets_data = BusinessAssets::new(
            4000.0, // Cash equivalent
            0, 
            0, 
            0
    ).expect("Valid assets");
    let cash_calculator = BusinessZakatCalculator::new(cash_assets_data)
        .with_hawl(true); // Ensure hawl is met

    let portfolio = ZakatPortfolio::new()
        .add(gold_asset)
        .add(cash_calculator);

    let result = portfolio.calculate_total(&config);
    assert!(result.errors.is_empty(), "Result should not have errors");

    // Verify Total Assets
    // Gold: 50 * 100 = 5000
    // Cash: 4000
    // Total: 9000
    assert_eq!(result.total_assets, dec!(9000.0), "Total assets should match");

    // Verify Zakat Due
    // 9000 * 0.025 = 225.0
    assert_eq!(result.total_zakat_due, dec!(225.0), "Total zakat should be calculated on aggregated sum");

    // Verify Individual Details updated
    for detail in result.details {
        assert!(detail.is_payable, "Component {:?} should be marked payable due to aggregation", detail.wealth_type);
        if let Some(reason) = detail.status_reason {
            assert!(reason.contains("Aggregation"), "Reason should mention aggregation");
        }
    }
}

#[test]
fn test_portfolio_no_aggregation_if_total_below_nisab() {
    // Scenario:
    // Gold Price: $100/g. Nisab = $8,500.
    // User has:
    // 1. 30g Gold = $3,000.
    // 2. Cash = $2,000.
    // Total = $5,000 < $8,500.
    // Zakat Due = 0.

    let config = ZakatConfig::new(100.0, 1.0).unwrap();
    
    let gold_asset = PreciousMetals::new(30.0, WealthType::Gold).unwrap();
    let cash_assets_data = BusinessAssets::new(2000.0, 0, 0, 0).expect("Valid");
    let cash_calculator = BusinessZakatCalculator::new(cash_assets_data)
        .with_hawl(true);

    let portfolio = ZakatPortfolio::new()
        .add(gold_asset)
        .add(cash_calculator);

    let result = portfolio.calculate_total(&config);
    assert!(result.errors.is_empty());

    assert_eq!(result.total_assets, dec!(5000.0));
    assert_eq!(result.total_zakat_due, dec!(0.0));
    
    for detail in result.details {
        assert!(!detail.is_payable);
    }
}
