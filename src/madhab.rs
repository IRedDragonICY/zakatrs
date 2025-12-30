
use serde::{Deserialize, Serialize};

/// Nisab standard for calculating the Zakat threshold on monetary wealth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NisabStandard {
    /// Use the gold Nisab (85g × gold_price)
    #[default]
    Gold,
    /// Use the silver Nisab (595g × silver_price)
    Silver,
    /// Use the lower of gold or silver Nisab - most beneficial for the poor
    LowerOfTwo,
}

/// Islamic school of thought (Madhab) for Zakat calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Madhab {
    #[default]
    Hanafi,
    Shafi,
    Maliki,
    Hanbali,
}

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZakatRules {
    pub nisab_standard: NisabStandard,
    pub jewelry_exempt: bool,
    pub trade_goods_rate: Decimal, // default 0.025
    pub agriculture_rates: (Decimal, Decimal, Decimal), // Rain, Irrigated, Mixed
}

impl Default for ZakatRules {
    fn default() -> Self {
        Self {
            nisab_standard: NisabStandard::default(),
            jewelry_exempt: true,
            trade_goods_rate: dec!(0.025),
            agriculture_rates: (dec!(0.10), dec!(0.05), dec!(0.075)),
        }
    }
}

/// Trait for providing Zakat calculation rules.
/// 
/// Implement this trait to create custom Zakat strategies beyond the standard Madhabs.
/// For example, a "Gregorian Tax Year" strategy or institution-specific rules.
pub trait ZakatStrategy: std::fmt::Debug + Send + Sync {
    /// Returns the rules that govern Zakat calculations.
    fn get_rules(&self) -> ZakatRules;
}

// ============ Implement ZakatStrategy for Madhab enum (preset helper) ============

impl ZakatStrategy for Madhab {
    fn get_rules(&self) -> ZakatRules {
        match self {
            Madhab::Hanafi => HanafiStrategy.get_rules(),
            Madhab::Shafi => ShafiStrategy.get_rules(),
            Madhab::Maliki => MalikiStrategy.get_rules(),
            Madhab::Hanbali => HanbaliStrategy.get_rules(),
        }
    }
}

// ============ Internal Strategy Implementations ============

#[derive(Debug)]
struct HanafiStrategy;
impl ZakatStrategy for HanafiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::LowerOfTwo,
            jewelry_exempt: false, // Hanafi views jewelry as wealth (Amwal Namiya)
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct ShafiStrategy;
impl ZakatStrategy for ShafiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::Gold,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct MalikiStrategy;
impl ZakatStrategy for MalikiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::Gold,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
struct HanbaliStrategy;
impl ZakatStrategy for HanbaliStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::LowerOfTwo,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}
