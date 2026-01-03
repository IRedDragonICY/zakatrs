
//! # Fiqh Compliance: Madhab Strategy Pattern
//!
//! ## Principle: Ikhtilaf al-Fuqaha (Valid Divergence)
//! This module implements the "Pluggable Strategy" pattern to honor valid juristic differences.
//! - **LowerOfTwo**: Hanafi view for Nisab (Gold/Silver), prioritizing benefit for the poor (*Anfa' lil-fuqara*).
//! - **Gold Standard**: Shafi'i/Maliki/Hanbali/Modern view (Qaradawi).
//! - **Jewelry Exemption**: Toggles between Obligatory (Hanafi) and Exempt (Jumhur/Majority).

use serde::{Deserialize, Serialize};

/// Nisab standard for calculating the Zakat threshold on monetary wealth.
/// - `Gold`: Use the gold Nisab (85g × gold_price).
/// - `Silver`: Use the silver Nisab (595g × silver_price).
/// - `LowerOfTwo`: Use the lower of gold or silver Nisab - most beneficial for the poor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
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
/// Each Madhab has different rules regarding Nisab standards and jewelry exemptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub enum Madhab {
    /// Hanafi Madhab - Uses LowerOfTwo Nisab standard, jewelry is zakatable.
    #[default]
    Hanafi,
    /// Shafi'i Madhab - Uses Gold Nisab standard, personal jewelry is exempt.
    Shafi,
    /// Maliki Madhab - Uses Gold Nisab standard, personal jewelry is exempt.
    Maliki,
    /// Hanbali Madhab - Uses LowerOfTwo Nisab standard, personal jewelry is exempt.
    Hanbali,
}

use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Rules that govern Zakat calculations for a specific Madhab.
/// Contains Nisab standard, jewelry exemption policy, and Zakat rates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct ZakatRules {
    /// The Nisab standard to use for determining Zakat eligibility.
    pub nisab_standard: NisabStandard,
    /// Whether personal jewelry is exempt from Zakat.
    pub jewelry_exempt: bool,
    /// Zakat rate for trade goods (default: 2.5% = 0.025).
    #[typeshare(serialized_as = "string")]
    pub trade_goods_rate: Decimal,
    /// Agriculture Zakat rates: (Rain-fed, Irrigated, Mixed).
    #[typeshare(skip)]
    pub agriculture_rates: (Decimal, Decimal, Decimal),
}

use crate::inputs::IntoZakatDecimal;

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

impl ZakatRules {
    /// Sets the Nisab standard.
    pub fn with_nisab_standard(mut self, standard: NisabStandard) -> Self {
        self.nisab_standard = standard;
        self
    }

    /// Sets whether jewelry is exempt.
    pub fn with_jewelry_exempt(mut self, exempt: bool) -> Self {
        self.jewelry_exempt = exempt;
        self
    }

    /// Sets the trade goods Zakat rate using a semantic decimal type.
    /// Accepts literals like `0.025` directly.
    pub fn with_trade_goods_rate(mut self, rate: impl IntoZakatDecimal) -> Self {
        if let Ok(rate) = rate.into_zakat_decimal() {
            self.trade_goods_rate = rate;
        }
        self
    }

    /// Sets the agriculture rates (Rain-fed, Irrigated, Mixed).
    /// Accepts literals directly.
    pub fn with_agriculture_rates(
        mut self, 
        rain: impl IntoZakatDecimal, 
        irrigated: impl IntoZakatDecimal, 
        mixed: impl IntoZakatDecimal
    ) -> Self {
        if let (Ok(r), Ok(i), Ok(m)) = (
            rain.into_zakat_decimal(), 
            irrigated.into_zakat_decimal(), 
            mixed.into_zakat_decimal()
        ) {
            self.agriculture_rates = (r, i, m);
        }
        self
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
