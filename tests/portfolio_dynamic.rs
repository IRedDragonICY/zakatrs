use zakat::portfolio::ZakatPortfolio;
use zakat::maal::business::BusinessZakat;
use zakat::config::ZakatConfig;
use rust_decimal_macros::dec;


#[test]
fn test_dynamic_portfolio_operations() {
    let mut portfolio = ZakatPortfolio::new();
    let asset1 = BusinessZakat::new().cash(dec!(10000)).label("Shop 1");
    let asset2 = BusinessZakat::new().cash(dec!(5000)).label("Shop 2");

    let id1 = portfolio.push(asset1);
    let (portfolio, id2) = portfolio.add_with_id(asset2);
    let mut portfolio = portfolio; 

    // Verify IDs are distinct
    assert_ne!(id1, id2);

    // Verify Get
    assert!(portfolio.get(id1).is_some());
    assert!(portfolio.get(id2).is_some());

    // Verify Remove
    let removed = portfolio.remove(id1);
    assert!(removed.is_some());
    assert!(portfolio.get(id1).is_none());
    assert!(portfolio.get(id2).is_some()); // id2 still there

    // Verify Replace
    let new_asset = BusinessZakat::new().cash(dec!(20000)).label("Shop 2 Updated");
    let replace_res = portfolio.replace(id2, new_asset);
    assert!(replace_res.is_ok());
    
    // Verify Replace
    // Note: Replacing an asset swaps the underlying object. Since the ID is internal to the asset,
    // the new asset will have a different ID. The old ID is no longer valid for lookup.
    let _new_asset = BusinessZakat::new().cash(dec!(20000)).label("Shop 2 Updated");
    
    // Check total
    let config = ZakatConfig::new().with_gold_price(100);
    let result = portfolio.calculate_total(&config);
    assert_eq!(result.successes()[0].net_assets, dec!(20000));
}
