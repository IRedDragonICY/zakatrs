use zakat_core::maal::precious_metals::{PreciousMetals, JewelryUsage, Gender};
use zakat_core::maal::livestock::{LivestockAssets, LivestockPrices, LivestockType};
use zakat_core::maal::agriculture::{AgricultureAssets, IrrigationMethod};
use zakat_core::config::ZakatConfig;
use zakat_core::madhab::Madhab;
use zakat_core::traits::CalculateZakat;
use rust_decimal_macros::dec;
use rust_decimal::Decimal;

#[test]
fn test_ring_with_diamond() {
    let config = ZakatConfig::new().with_gold_price(100.0);
    
    // 5g Ring, 2g Diamond.
    // Net Gold Weight = 3g.
    // Value = 3g * $100 = $300.
    // Nisab 85g * $100 = $8500.
    // Total < Nisab -> Not Payable.
    
    let ring = PreciousMetals::new()
        .weight(5.0)
        .with_stones(2.0)
        .metal_type(zakat_core::types::WealthType::Gold)
        .hawl(true);
        
    let details = ring.calculate_zakat(&config).unwrap();
    
    assert!(!details.is_payable);
    assert_eq!(details.total_assets, dec!(300.0));
    
    // Check trace for deduction
    let trace_str = format!("{:?}", details.calculation_trace);
    assert!(trace_str.contains("Gemstones Deduction"));
}

#[test]
fn test_male_gold_jewelry() {
    let config = ZakatConfig::new()
        .with_gold_price(100.0)
        .with_madhab(Madhab::Shafi); // Shafi usually exempts personal jewelry
        
    // 100g Gold (Above Nisab). Personal Usage. Male.
    // Should be PAYABLE because Gold is Haram for men.
    
    let gold = PreciousMetals::new()
        .weight(100.0)
        .metal_type(zakat_core::types::WealthType::Gold)
        .usage(JewelryUsage::PersonalUse)
        .gender(Gender::Male)
        .hawl(true);
        
    let details = gold.calculate_zakat(&config).unwrap();
    
    assert!(details.is_payable);
    assert!(details.zakat_due > Decimal::ZERO);
    
    // Check trace for male gold note
    let trace_str = format!("{:?}", details.calculation_trace);
    assert!(trace_str.contains("Gold held by male is not exempt"));
}

#[test]
fn test_female_gold_jewelry_shafi() {
    let config = ZakatConfig::new()
        .with_gold_price(100.0)
        .with_madhab(Madhab::Shafi);
        
    // 100g Gold. Personal Usage. Female.
    // Should be EXEMPT.
    
    let gold = PreciousMetals::new()
        .weight(100.0)
        .metal_type(zakat_core::types::WealthType::Gold)
        .usage(JewelryUsage::PersonalUse)
        .gender(Gender::Female)
        .hawl(true);
        
    let details = gold.calculate_zakat(&config).unwrap();
    
    assert!(!details.is_payable);
    assert!(details.status_reason.unwrap().contains("Exempt"));
}

#[test]
fn test_working_camels() {
    let prices = LivestockPrices::new()
        .camel_price(1000.0)
        .sheep_price(100.0);
        
    // 30 Camels. Usually leads to 1 Bint Makhad ($500).
    // But Working -> Exempt.
    
    let camels = LivestockAssets::new()
        .count(30)
        .animal_type(LivestockType::Camel)
        .prices(prices)
        .working_animal(true)
        .hawl(true);
        
    let details = camels.calculate_zakat(&ZakatConfig::default()).unwrap();
    
    assert!(!details.is_payable);
    assert!(details.status_reason.unwrap().contains("Working animals"));
}

#[test]
fn test_agriculture_expenses() {
    let config = ZakatConfig::default(); // Nisab 653kg
    
    // Harvest 1000kg (Above 653kg).
    // Price $1/kg -> Gross $1000.
    // Costs $400.
    // Net Value $600.
    // Nisab Value 653kg * $1 = $653.
    // Net ($600) < Nisab ($653) -> NOT Payable.
    
    let agri = AgricultureAssets::new()
        .harvest_weight(1000.0)
        .price(1.0)
        .costs(400.0) // Significant costs
        .irrigation(IrrigationMethod::Rain)
        .hawl(true);
        
    let details = agri.calculate_zakat(&config).unwrap();
    
    assert!(!details.is_payable);
    
    // Check trace
    let trace_str = format!("{:?}", details.calculation_trace);
    assert!(trace_str.contains("step-deduct-costs"));
    
    // Test Payable Case
    // Costs $200 -> Net $800 > $653 -> Payable
    let agri_payable = AgricultureAssets::new()
        .harvest_weight(1000.0)
        .price(1.0)
        .costs(200.0)
        .irrigation(IrrigationMethod::Rain)
        .hawl(true);
        
    let details_p = agri_payable.calculate_zakat(&config).unwrap();
    assert!(details_p.is_payable);
    // Rate 10% on Net $800 = $80
    assert_eq!(details_p.zakat_due, dec!(80.0));
}
