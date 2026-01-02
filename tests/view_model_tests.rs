use zakat::types::{ZakatDetails, WealthType};
use zakat::config::ZakatConfig;
use rust_decimal_macros::dec;

#[test]
fn test_explanation_model_formatting() {
    let config = ZakatConfig::default(); // Defaults to EnUS, USD
    
    // Mock a ZakatDetails
    let details = ZakatDetails::new(
        dec!(10000), // Total
        dec!(0),     // Liabilities
        dec!(5000),  // Nisab
        dec!(0.025), // Rate
        WealthType::Business,
    );

    let explanation = details.to_explanation(&config);

    // Verify Formatting
    let total = explanation.formatted_total.replace("\u{00A0}", " ");
    let due = explanation.formatted_due.replace("\u{00A0}", " ");
    
    println!("DEBUG TOTAL: '{}'", total);
    println!("DEBUG DUE: '{}'", due);

    // Expected output based on current i18n implementation and Decimal scaling
    // Total: $10,000 (Scale 0)
    assert!(total.contains("$") && total.contains("10,000"));
    // Due: $250.000 (Scale 3 from 0.025 calculation)
    assert!(due.contains("$") && due.contains("250."));
    
    assert_eq!(explanation.currency_code, "USD");
    assert_eq!(explanation.nisab_progress, 1.0);
    assert_eq!(explanation.status, "PAYABLE");
}

#[test]
fn test_explanation_nisab_progress() {
    let config = ZakatConfig::default();
    
    let details = ZakatDetails::new(
        dec!(2500),  // Total
        dec!(0),
        dec!(5000),  // Nisab
        dec!(0.025),
        WealthType::Business,
    );
    let explanation = details.to_explanation(&config);

    // 2500 / 5000 = 0.5
    assert_eq!(explanation.nisab_progress, 0.5);
    assert_eq!(explanation.status, "EXEMPT");
}
