use zakat::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn test_dx_prelude_and_ergonomics() {
    // 1. Verify Prelude Imports work (no specific imports needed beyond zakat::prelude::*)
    
    // 2. Verify Ergonomic Inputs (Integers)
    // ZakatConfig: Passing integers
    let config = ZakatConfig::new(100, 1) // i32, i32
        .with_gold_nisab(85); // i32
        
    assert_eq!(config.gold_price_per_gram, dec!(100.0));
    assert_eq!(config.silver_price_per_gram, dec!(1.0));
    
    // BusinessAssets: Passing integers
    let business = BusinessAssets::new(
        10000, // cash: i32
        5000,  // inventory: i32
        0,     // receivables: i32
        1000   // debt: i32
    ).expect("Valid assets");
    
    assert_eq!(business.cash_on_hand, dec!(10000.0));
    
    // Income: Passing i32
    let income = IncomeZakatCalculator::new(
        12000, // total_income: i32
        4000,   // basic_expenses: i32
        IncomeCalculationMethod::Net
    ).expect("Config valid");
    
    let res = income.with_debt_due_now(500).calculate_zakat(&config).unwrap();
    // Net: 12000 - 4000 - 500 = 7500.
    // Nisab: 85 * 100 = 8500.
    // Not payable.
    assert!(!res.is_payable);
    
    // Precious Metals
    let gold = PreciousMetals::new(
        85, // weight: i32
        WealthType::Gold
    ).expect("Valid");
    
    // 85g >= 85g. Payable.
    let gold_res = gold.calculate_zakat(&config).unwrap();
    assert!(gold_res.is_payable);
}
