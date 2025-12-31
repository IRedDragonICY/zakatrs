use zakat::prelude::*;

use zakat::types::ZakatError;

#[test]
fn test_labeling_workflow() {
    let config = ZakatConfig::new().with_gold_price(100);

    let business_a = BusinessZakat::new()
        .cash(10000)
        .label("Shop A");

    let business_b = BusinessZakat::new()
        .cash(500)
        .label("Shop B");

    let details_a = business_a.calculate_zakat(&config).unwrap();
    let details_b = business_b.calculate_zakat(&config).unwrap();

    assert_eq!(details_a.label, Some("Shop A".to_string()));
    assert_eq!(details_b.label, Some("Shop B".to_string()));
}

#[test]
fn test_sanitization_negative_weight() {
    let config = ZakatConfig::default();
    let res = PreciousMetals::new()
        .weight(-50.0)
        .metal_type(WealthType::Gold)
        .calculate_zakat(&config);
    
    assert!(res.is_err());
    if let Err(ZakatError::InvalidInput { reason: msg, .. }) = res {
        assert!(msg.contains("must be non-negative"));
    } else {
        panic!("Expected InvalidInput error, got {:?}", res);
    }
}

#[test]
fn test_sanitization_business_negative() {
    let config = ZakatConfig::default();
    let res = BusinessZakat::new()
        .cash(-100)
        .calculate_zakat(&config);
    assert!(res.is_err());
}

#[test]
fn test_sanitization_income_negative() {
    let config = ZakatConfig::default();
    let res = IncomeZakatCalculator::new()
        .income(-1000)
        .expenses(0)
        .method(IncomeCalculationMethod::Gross)
        .calculate_zakat(&config);
    assert!(res.is_err());
}

#[test]
fn test_sanitization_investment_negative() {
    let config = ZakatConfig::default();
    let res = InvestmentAssets::new()
        .value(-500)
        .kind(InvestmentType::Stock)
        .calculate_zakat(&config);
    assert!(res.is_err());
}
