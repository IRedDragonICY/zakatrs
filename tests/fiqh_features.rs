use zakat::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{NaiveDate, Duration, Local};

#[test]
fn test_hawl_tracking_logic() {
    let now = Local::now().date_naive();
    
    // Case 1: Acquired 350 days ago (Less than 354) -> Should NOT be payable
    let date_short = now - Duration::days(350);
    
    let config = ZakatConfig { 
        gold_price_per_gram: dec!(100), 
        ..Default::default() 
    };

    let business_short = BusinessZakat::new()
        .cash(20_000.0) // Above Nisab ($8,500)
        .acquired_on(date_short);
        
    let result_short = business_short.calculate_zakat(&config).unwrap();
    assert!(!result_short.is_payable, "Should not be payable if held < 354 days");
    assert!(format!("{:?}", result_short.status_reason).contains("Hawl"));

    // Case 2: Acquired 355 days ago (More than 354) -> Should be payable
    let date_long = now - Duration::days(355);
    
    let business_long = BusinessZakat::new()
        .cash(20_000.0)
        .acquired_on(date_long);
        
    let result_long = business_long.calculate_zakat(&config).unwrap();
    assert!(result_long.is_payable, "Should be payable if held > 354 days");
}

#[test]
fn test_investment_purification() {
    let config = ZakatConfig { 
        gold_price_per_gram: dec!(100), 
        ..Default::default() 
    };
    
    // Investment: $10,000
    // Purification: 5% (0.05)
    // Impure Amount: $500
    // Net Zakatable: $9,500
    // Nisab (85g * 100): $8,500
    // Zakat Due (2.5% of $9,500): $237.50
    
    let investment = InvestmentAssets::new()
        .value(10_000.0)
        .purify(0.05)
        .hawl(true); // Manually set Hawl satisfied
        
    let result = investment.calculate_zakat(&config).unwrap();
    
    assert!(result.is_payable);
    assert_eq!(result.net_assets, dec!(9500.0));
    assert_eq!(result.zakat_due, dec!(237.50));
    
    // Verify trace contains purification
    let trace_str = format!("{:?}", result.calculation_trace);
    assert!(trace_str.contains("Purification Rate (Tathir)"));
    assert!(trace_str.contains("Impure Amount Deducted"));
}

#[test]
fn test_hawl_override_priority() {
    let now = Local::now().date_naive();
    
    // If acquisition_date is set, it should override manually setting hawl(true)
    let date_short = now - Duration::days(100);
    
    let config = ZakatConfig { 
        gold_price_per_gram: dec!(100), 
        ..Default::default() 
    };
    
    let business = BusinessZakat::new()
        .cash(20_000.0)
        .acquired_on(date_short)
        .hawl(true); // Attempt to manual override (should be ignored)
        
    let result = business.calculate_zakat(&config).unwrap();
    assert!(!result.is_payable, "Acquisition date should take precedence over manual boolean");
}
