
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

pub trait MadhabStrategy {
    fn nisab_standard(&self) -> NisabStandard;
    
    /// Determines if personal jewelry is exempt from Zakat.
    /// Hanafi: Not exempt (Payable).
    /// Shafi/Maliki/Hanbali: Exempt if for personal use and within moderation.
    fn is_jewelry_exempt(&self) -> bool {
        true // Default to exempt (Shafi/Maliki/Hanbali)
    }
}

pub struct HanafiStrategy;
impl MadhabStrategy for HanafiStrategy {
    fn nisab_standard(&self) -> NisabStandard {
        NisabStandard::LowerOfTwo
    }

    fn is_jewelry_exempt(&self) -> bool {
        false // Hanafi views jewelry as wealth (Amwal Namiya), so it is payable.
    }
}

pub struct ShafiStrategy;
impl MadhabStrategy for ShafiStrategy {
    fn nisab_standard(&self) -> NisabStandard {
        NisabStandard::Gold
    }
}

pub struct MalikiStrategy;
impl MadhabStrategy for MalikiStrategy {
    fn nisab_standard(&self) -> NisabStandard {
        NisabStandard::Gold
    }
}

pub struct HanbaliStrategy;
impl MadhabStrategy for HanbaliStrategy {
    fn nisab_standard(&self) -> NisabStandard {
        NisabStandard::LowerOfTwo
    }
}
