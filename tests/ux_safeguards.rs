use zakat::prelude::*;
use rust_decimal_macros::dec;
use zakat::types::ZakatError;

#[test]
fn test_labeling_workflow() {
    let config = ZakatConfig {
        gold_price_per_gram: dec!(100.0),
        ..Default::default()
    };

    let business_a_assets = BusinessAssets::new(dec!(10000), dec!(0), dec!(0), dec!(0)).unwrap();
    let business_a = BusinessZakatCalculator::new(business_a_assets, &config)
        .unwrap()
        .with_label("Shop A");

    let business_b_assets = BusinessAssets::new(dec!(500), dec!(0), dec!(0), dec!(0)).unwrap();
    let business_b = BusinessZakatCalculator::new(business_b_assets, &config)
        .unwrap()
        .with_label("Shop B");

    let details_a = business_a.calculate_zakat().unwrap();
    let details_b = business_b.calculate_zakat().unwrap();

    assert_eq!(details_a.label, Some("Shop A".to_string()));
    assert_eq!(details_b.label, Some("Shop B".to_string()));
}

#[test]
fn test_sanitization_negative_weight() {
    let config = ZakatConfig::default();
    let res = PreciousMetal::new(dec!(-50.0), WealthType::Gold, &config);
    
    assert!(res.is_err());
    if let Err(ZakatError::InvalidInput(msg)) = res {
        assert_eq!(msg, "Weight must be non-negative");
    } else {
        panic!("Expected InvalidInput error");
    }
}

#[test]
fn test_sanitization_business_negative() {
    let res = BusinessAssets::new(dec!(-100), dec!(0), dec!(0), dec!(0));
    assert!(res.is_err());
}

#[test]
fn test_sanitization_income_negative() {
    let config = ZakatConfig::default();
    let res = IncomeZakatCalculator::new(dec!(-1000), dec!(0), IncomeCalculationMethod::Gross, &config);
    assert!(res.is_err());
}

#[test]
fn test_sanitization_investment_negative() {
    let config = ZakatConfig::default();
    let res = InvestmentAssets::new(dec!(-500), InvestmentType::Stock, &config);
    assert!(res.is_err());
}
