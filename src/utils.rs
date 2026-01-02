use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Converts grams to Tola.
/// 1 Tola is approximately 11.66 grams.
pub fn grams_to_tola(grams: Decimal) -> Decimal {
    let tola_in_grams = dec!(11.66);
    grams / tola_in_grams
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeightUnit {
    #[default]
    Grams,
    TroyOunce, // 31.1034768 grams
    Tola,      // 11.66 grams
}

impl WeightUnit {
    pub fn to_grams(&self, value: Decimal) -> Decimal {
        match self {
            WeightUnit::Grams => value,
            WeightUnit::TroyOunce => value * dec!(31.1034768),
            WeightUnit::Tola => value * dec!(11.66),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grams_to_tola() {
        let grams = dec!(11.66);
        let tola = grams_to_tola(grams);
        assert_eq!(tola, dec!(1));
    }

    #[test]
    fn test_weight_conversions() {
        // Tola to Grams
        assert_eq!(WeightUnit::Tola.to_grams(dec!(1)), dec!(11.66));
        // Ounce to Grams
        assert_eq!(WeightUnit::TroyOunce.to_grams(dec!(1)), dec!(31.1034768));
        // Grams to Grams
        assert_eq!(WeightUnit::Grams.to_grams(dec!(100)), dec!(100));
    }
}
