//! Tests for the pluggable ZakatStrategy pattern.
//!
//! This demonstrates how users can create custom Zakat strategies
//! that differ from the traditional Madhab presets.

use std::sync::Arc;
use rust_decimal_macros::dec;
use zakat::prelude::*;

/// A custom strategy for Gregorian Tax Year calculations.
/// Uses 2.577% instead of the standard 2.5% rate.
#[derive(Debug, Clone)]
struct CustomGregorianStrategy;

impl ZakatStrategy for CustomGregorianStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::Gold,
            jewelry_exempt: false,
            trade_goods_rate: dec!(0.02577), // 2.577%
            agriculture_rates: (dec!(0.10), dec!(0.05), dec!(0.075)),
        }
    }
}

#[test]
fn test_custom_strategy_with_with_madhab() {
    // Test using with_madhab() which accepts any impl ZakatStrategy
    let config = ZakatConfig::new()
        .with_gold_price(dec!(100.0))
        .with_madhab(CustomGregorianStrategy);
    
    // Verify the strategy is applied
    let rules = config.strategy.get_rules();
    assert_eq!(rules.trade_goods_rate, dec!(0.02577));
    assert!(!rules.jewelry_exempt);
}

#[test]
fn test_custom_strategy_with_arc() {
    // Test using with_strategy() which accepts Arc<dyn ZakatStrategy>
    let strategy = Arc::new(CustomGregorianStrategy);
    let config = ZakatConfig::new()
        .with_gold_price(dec!(100.0))
        .with_strategy(strategy);
    
    // Verify the strategy is applied
    let rules = config.strategy.get_rules();
    assert_eq!(rules.trade_goods_rate, dec!(0.02577));
}

#[test]
fn test_preset_madhab_still_works() {
    // Verify backward compatibility - Madhab presets still work
    let config = ZakatConfig::new()
        .with_gold_price(dec!(100.0))
        .with_madhab(Madhab::Shafi);
    
    let rules = config.strategy.get_rules();
    assert!(rules.jewelry_exempt);
    assert_eq!(rules.nisab_standard, NisabStandard::Gold);
}

#[test]
fn test_custom_strategy_affects_calculation() {
    // For precious metals, the strategy's jewelry_exempt rule affects calculation
    let config = ZakatConfig::new()
        .with_gold_price(dec!(100.0))
        .with_madhab(CustomGregorianStrategy);
    
    // CustomGregorianStrategy has jewelry_exempt = false
    // So personal jewelry should be zakatable
    let gold = PreciousMetals::new()
        .weight(dec!(100.0))
        .metal_type(WealthType::Gold)
        .usage(zakat::maal::precious_metals::JewelryUsage::PersonalUse)
        .hawl(true);
    
    let result = gold.calculate_zakat(&config).unwrap();
    
    // Since jewelry_exempt is false in our custom strategy,
    // personal jewelry SHOULD be payable (100g > 85g nisab)
    assert!(result.is_payable);
}

#[test]
fn test_default_strategy_is_hanafi() {
    let config = ZakatConfig::new();
    let rules = config.strategy.get_rules();
    
    // Hanafi uses LowerOfTwo and jewelry is NOT exempt
    assert_eq!(rules.nisab_standard, NisabStandard::LowerOfTwo);
    assert!(!rules.jewelry_exempt);
}
