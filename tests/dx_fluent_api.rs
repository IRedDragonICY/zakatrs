use zakat::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn test_dx_prelude_and_ergonomics() {
    // 1. Verify Prelude Imports work (no specific imports needed beyond zakat::prelude::*)
    
    // 2. Verify Ergonomic Inputs (Integers)
    // ZakatConfig: Passing integers
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_silver_price(1)
        .with_gold_nisab(85); // i32
        
    assert_eq!(config.gold_price_per_gram, dec!(100.0));
    assert_eq!(config.silver_price_per_gram, dec!(1.0));
    
    // BusinessAssets: Passing integers
    let business = BusinessZakat::new()
        .cash(10000)
        .inventory(5000)
        .receivables(0)
        .liabilities(1000);
    
    assert_eq!(business.cash_on_hand, dec!(10000.0));
    
    // Income: Passing i32
    let income = IncomeZakatCalculator::new()
        .income(12000) // total_income: i32
        .expenses(4000)   // basic_expenses: i32
        .method(IncomeCalculationMethod::Net);
    
    let res = income.debt(500).calculate_zakat(&config).unwrap();
    // Net: 12000 - 4000 - 500 = 7500.
    // Nisab: 85 * 100 = 8500.
    // Not payable.
    assert!(!res.is_payable);
    
    // Precious Metals
    let gold = PreciousMetals::new()
        .weight(85) // weight: i32
        .metal_type(WealthType::Gold);
    
    // 85g >= 85g. Payable.
    let gold_res = gold.calculate_zakat(&config).unwrap();
    assert!(gold_res.is_payable);
}

#[test]
fn test_error_context_labels() {
    let config = ZakatConfig::default();
    
    // Test Invalid Input with Label
    let res = PreciousMetals::new()
        .weight(-50.0) // Invalid
        .metal_type(WealthType::Gold)
        .label("Grandma's Broken Necklace")
        .calculate_zakat(&config);
        
    assert!(res.is_err());
    let err = res.err().unwrap();
    
    // Check if the error string contains the label
    let err_str = err.report(&zakat::i18n::default_translator());
    println!("Error String: {}", err_str);
    assert!(err_str.contains("Grandma's Broken Necklace"), "Error message should contain asset label");
    assert!(err_str.contains("Value must be non-negative"));
}
