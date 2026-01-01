use zakat::prelude::*;

#[test]
fn test_explain_output_format() {
    let config = ZakatConfig::new()
        .with_gold_price(100)
        .with_gold_nisab(85);

    let gold = PreciousMetals::new()
        .weight(100)
        .metal_type(WealthType::Gold)
        .label("My Gold");

    let result = gold.calculate_zakat(&config).unwrap();
    let explanation = result.explain(&config.translator);

    println!("{}", explanation);

    // Verify key elements
    // Note: Specific assets might use "Total Value" or "Weight" instead of generic "Total Assets"
    assert!(explanation.contains("My Gold (Gold)"));
    assert!(explanation.contains("Status: PAYABLE"));
    assert!(explanation.contains("Amount Due: $250.00")); // 100 * 100 * 0.025 = 250
    
    // Check for specific trace steps we know exist for Gold
    assert!(explanation.contains("Weight (grams)")); 
    assert!(explanation.contains("Total Value"));
}
