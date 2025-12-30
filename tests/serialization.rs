use zakat::prelude::*;
use zakat::types::WealthType;
use zakat::config::ZakatConfig;
use zakat::ZakatPortfolio;
use zakat::assets::PortfolioItem;
use rust_decimal_macros::dec;
use serde_json;

#[test]
fn test_portfolio_serialization() {
    // 1. Create a portfolio with diverse assets
    let mut portfolio = ZakatPortfolio::new();
    
    // Business Asset
    portfolio = portfolio.add(
        BusinessZakat::new()
            .cash(dec!(10000))
            .inventory(dec!(5000))
            .label("Main Store")
    );
    
    // Gold Asset
    portfolio = portfolio.add(
        PreciousMetals::new()
            .weight(100)
            .metal_type(WealthType::Gold)
            .label("Gold Reserve")
    );
    
    // Livestock Asset
    portfolio = portfolio.add(
        LivestockAssets::new()
            .count(40)
            .animal_type(LivestockType::Sheep)
            .prices(LivestockPrices::new().sheep_price(dec!(100)))
            .label("Sheep Herd")
    );

    // 2. Serialize to JSON
    let json = serde_json::to_string_pretty(&portfolio).expect("Failed to serialize portfolio");
    println!("Serialized JSON:\n{}", json);
    
    // Verify JSON contains key fields
    assert!(json.contains("Main Store"));
    assert!(json.contains("Gold Reserve"));
    assert!(json.contains("type")); // Tag for enum
    assert!(json.contains("Business"));
    
    // 3. Deserialize back
    let deserialized: ZakatPortfolio = serde_json::from_str(&json).expect("Failed to deserialize portfolio");
    
    // 4. Verify content matches
    let items = deserialized.get_items();
    assert_eq!(items.len(), 3);
    
    // 5. Verify calculation works on deserialized object
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_silver_price(1);
        
    let report = deserialized.calculate_total(&config);
    assert!(report.is_clean());
    
    assert_eq!(report.total_assets, dec!(15000) + dec!(10000) + dec!(4000)); // 15k business, 10k gold, 4k sheep
    // Total 29000.
    
    // 2.5% of 29000 = 725
    // Business: 15000 * 0.025 = 375.
    // Gold: 10000 * 0.025 = 250.
    // Sheep: 40 sheep -> 1 sheep * 100 = 100.
    // Total Zakat = 375 + 250 + 100 = 725.
    assert_eq!(report.total_zakat_due, dec!(725.0));
}

#[test]
fn test_individual_asset_serialization() {
    let asset = BusinessZakat::new().cash(dec!(500));
    let json = serde_json::to_string(&asset).unwrap();
    let back: BusinessZakat = serde_json::from_str(&json).unwrap();
    assert_eq!(back.cash_on_hand, dec!(500));
    
    // As PortfolioItem
    let item: PortfolioItem = asset.into();
    let json_item = serde_json::to_string(&item).unwrap();
    assert!(json_item.contains("\"type\":\"Business\""));
    let back_item: PortfolioItem = serde_json::from_str(&json_item).unwrap();
    
    if let PortfolioItem::Business(b) = back_item {
        assert_eq!(b.cash_on_hand, dec!(500));
    } else {
        panic!("Wrong variant deserialized");
    }
}
