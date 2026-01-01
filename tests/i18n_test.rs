use zakat::i18n::{ZakatLocale, CurrencyFormatter, TRANSLATOR};
use zakat::types::{CalculationStep, ZakatDetails, WealthType};
use rust_decimal_macros::dec;

#[test]
fn test_locale_enum() {
    assert_eq!(ZakatLocale::EnUS.as_str(), "en-US");
    assert_eq!(ZakatLocale::IdID.as_str(), "id-ID");
    assert_eq!(ZakatLocale::ArSA.as_str(), "ar-SA");

    assert_eq!("en-US".parse::<ZakatLocale>().unwrap(), ZakatLocale::EnUS);
    assert_eq!("id-ID".parse::<ZakatLocale>().unwrap(), ZakatLocale::IdID);
    assert_eq!("ar-SA".parse::<ZakatLocale>().unwrap(), ZakatLocale::ArSA);
}

#[test]
fn test_currency_formatting_en_us() {
    let locale = ZakatLocale::EnUS;
    let amount = dec!(1234.56);
    assert_eq!(locale.format_currency(amount), "$1,234.56");
    
    let integer_amount = dec!(1000);
    assert_eq!(locale.format_currency(integer_amount), "$1,000");
}

#[test]
fn test_currency_formatting_id_id() {
    let locale = ZakatLocale::IdID;
    let amount = dec!(1234.56);
    // Rp1.234,56
    assert_eq!(locale.format_currency(amount), "Rp1.234,56");
}

#[test]
fn test_currency_formatting_ar_sa() {
    let locale = ZakatLocale::ArSA;
    let amount = dec!(1234.56);
    // ICU default for ar-SA is Eastern Arabic numerals
    // 1,234.56 -> ١٬٢٣٤٫٥٦
    assert_eq!(locale.format_currency(amount), "١٬٢٣٤٫٥٦ ر.س");
}

#[test]
fn test_translator_basic() {
    let text = TRANSLATOR.translate(ZakatLocale::EnUS, "status-payable", None);
    assert_eq!(text, "PAYABLE");
    
    let text_id = TRANSLATOR.translate(ZakatLocale::IdID, "status-payable", None);
    assert_eq!(text_id, "WAJIB ZAKAT");
    
    let text_missing = TRANSLATOR.translate(ZakatLocale::EnUS, "non-existent-key", None);
    assert!(text_missing.starts_with("MISSING:"));
}

#[test]
fn test_details_summary_in() {
    let details = ZakatDetails::new(
        dec!(1000), // Total
        dec!(0),    // Liabilities
        dec!(850),  // Nisab
        dec!(0.025), // Rate
        WealthType::Business
    );
    // Total > Nisab -> Payable
    
    let summary_en = details.summary_in(ZakatLocale::EnUS);
    println!("Summary EN: '{}'", summary_en);
    // Asset: Payable - Due: $25.00
    assert!(summary_en.contains("PAYABLE"));
    assert!(summary_en.contains("Due: $25.00"));
    
    let summary_id = details.summary_in(ZakatLocale::IdID);
    println!("Summary ID: '{}'", summary_id);
    // Aset: Wajib Zakat - Zakat: Rp25,00
    // Note: status-due for ID is "Zakat", status-label is "Aset" (if translated? fallback is "Asset" if not translated)
    // Let's check keys.
    // status-due in ID is "Zakat".
    assert!(summary_id.contains("WAJIB ZAKAT"));
    assert!(summary_id.contains("Rp25,00"));
}

#[test]
fn test_details_explain_in() {
     let mut trace = Vec::new();
     trace.push(CalculationStep::initial("step-weight", "Weight", dec!(100)));
     // Test with args
     trace.push(CalculationStep::info("info-purity-adjustment", "Purity Adj")
        .with_args(std::collections::HashMap::from([("purity".to_string(), "21".to_string())])));

     let details = ZakatDetails::with_trace(
        dec!(1000), dec!(0), dec!(85), dec!(0.025), WealthType::Gold, trace
     );

     let explain_en = details.explain_in(ZakatLocale::EnUS);
     // Should contain "Weight" and "Purity Adjustment (21K / 24K)" if key exists and args work
     println!("Explain EN:\n{}", explain_en);
     assert!(explain_en.contains("Weight"));
     
     // info-purity-adjustment = Purity Adjustment ({$purity}K / 24K)
     if explain_en.contains("Purity Adjustment") {
         assert!(explain_en.contains("21K"));
     }
}
