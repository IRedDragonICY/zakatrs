use zakat::maal::precious_metals::PreciousMetals;
use zakat::traits::CalculateZakat;
use zakat::config::ZakatConfig;

#[test]
fn test_safe_setters_no_panic() {
    // This test verifies that passing invalid strings to fluent setters
    // does NOT cause a panic, but instead collects errors appropriately.
    
    // 1. PreciousMetals
    // Passing "invalid" should not panic
    let gold = PreciousMetals::gold("invalid")
        .debt("also_invalid");
        
    // 2. Calculation should return error
    let config = ZakatConfig::default();
    let result = gold.calculate_zakat(&config);
    
    // 3. Verify error kind
    assert!(result.is_err());
    let err = result.err().unwrap();
    
    println!("Caught expected error: {:?}", err);
    
    // We expect InvalidInput or MultipleErrors
    match err {
        zakat::types::ZakatError::MultipleErrors(errors) => {
             // Expecting errors for weight and debt
             assert!(errors.len() >= 1);
        },
        zakat::types::ZakatError::InvalidInput { field, .. } => {
            // If only one was caught/returned (first failure)
            println!("Single input error on field: {}", field);
        }
        _ => panic!("Unexpected error type: {:?}", err),
    }
}
