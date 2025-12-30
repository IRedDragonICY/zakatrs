
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

impl Madhab {
    pub fn strategy(&self) -> Box<dyn MadhabStrategy> {
        match self {
            Madhab::Hanafi => Box::new(HanafiStrategy),
            Madhab::Shafi => Box::new(ShafiStrategy),
            Madhab::Maliki => Box::new(MalikiStrategy),
            Madhab::Hanbali => Box::new(HanbaliStrategy),
        }
    }
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

pub trait MadhabStrategy {
    fn get_rules(&self) -> ZakatRules;
}

pub struct HanafiStrategy;
impl MadhabStrategy for HanafiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::LowerOfTwo,
            jewelry_exempt: false, // Hanafi views jewelry as wealth (Amwal Namiya)
            ..Default::default()
        }
    }
}

pub struct ShafiStrategy;
impl MadhabStrategy for ShafiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::Gold,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}

pub struct MalikiStrategy;
impl MadhabStrategy for MalikiStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::Gold,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}

pub struct HanbaliStrategy;
impl MadhabStrategy for HanbaliStrategy {
    fn get_rules(&self) -> ZakatRules {
        ZakatRules {
            nisab_standard: NisabStandard::LowerOfTwo,
            jewelry_exempt: true,
            ..Default::default()
        }
    }
}
