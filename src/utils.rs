use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Converts grams to Tola.
/// 1 Tola is approximately 11.66 grams.
pub fn grams_to_tola(grams: Decimal) -> Decimal {
    let tola_in_grams = dec!(11.66);
    grams / tola_in_grams
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grams_to_tola() {
        let grams = dec!(11.66);
        let tola = grams_to_tola(grams);
        assert_eq!(tola, dec!(1.0));
    }
}
