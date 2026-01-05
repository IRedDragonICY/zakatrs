use zakat_core::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn test_portfolio_convenience_methods() {
    let portfolio = ZakatPortfolio::new()
        .add_cash(10_000, "Savings")
        .add_gold(100, 0.999) // 24Kish
        .add_business(5000, 5000, 1000, 500);

    let items = portfolio.get_items();
    assert_eq!(items.len(), 3);

    // Verify Cash
    let _cash_item = portfolio.get_by_label("Savings").expect("Cash item found");
    // We can't easily downcast to check exact fields without exposing more, 
    // but we can check if it calculates.
}

#[test]
fn test_business_simple_constructor() {
    let biz = BusinessZakat::simple(5000, 3000);
    // Verify defaults
    // Private fields can't be accessed directly in integration test unless public
    // But we can verify calculation result.
    
    let config = ZakatConfig::test_default();
    let result = biz.calculate_zakat(&config).unwrap();
    
    // Net = 5000 + 3000 = 8000. 
    // Below nisab (85 * 85 = 7225). Payable?
    // Wait, 85g gold * $85/g = $7225.
    // 8000 > 7225. Yes payable.
    assert!(result.is_payable);
    assert_eq!(result.net_assets, dec!(8000));
}
