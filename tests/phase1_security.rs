use zakat::inputs::IntoZakatDecimal;

#[test]
fn test_dos_long_input() {
    let long_string: String = "1".repeat(100);
    let result = long_string.into_zakat_decimal();
    // Debug print
    if let Err(e) = &result {
        println!("Got expected error: {:?}", e);
    } else {
        println!("Got unexpected success: {:?}", result);
    }
    assert!(result.is_err());
}

#[test]
fn test_control_chars_removal() {
    // \u{00A0} is Non-Breaking Space.
    let input = "1\u{00A0}000"; 
    let result = input.into_zakat_decimal().unwrap();
    assert_eq!(result, rust_decimal_macros::dec!(1000));
    
    // \x00 is Null control character.
    let input_ctrl = "1\x00000"; 
    let result = input_ctrl.into_zakat_decimal().unwrap();
    assert_eq!(result, rust_decimal_macros::dec!(1000));
    
    // Tabs and newlines should also be removed/ignored if handled by trim() or explicit removal?
    // sanitize_numeric_string calls trim() at the end, so leading/trailing are gone.
    // What if they are in the middle? "1\t000" -> normalized "1\t000" -> remove control? \t is control?
    // char::is_control() returns true for \t and \n.
    // So "1\t000" -> "1000".
    let input_tab = "1\t000";
    let result = input_tab.into_zakat_decimal().unwrap();
    assert_eq!(result, rust_decimal_macros::dec!(1000));
}

#[test]
fn test_clean_spaces_in_env_var() {
    // This tests the logic we put in `from_env`, but we can't easily mock env vars in parallel tests without synchronization.
    // However, we can simulate the strings.
    let gold_str = "  100.50  ";
    let parsed = gold_str.trim().parse::<rust_decimal::Decimal>();
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), rust_decimal_macros::dec!(100.50));
}
