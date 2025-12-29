use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};

/// Calculates Zakat Fitrah.
///
/// # Arguments
///
/// * `person_count` - Number of people to pay for.
/// * `price_per_unit` - Price of the staple food per unit (kg or liter) defined by the local authority.
/// * `unit_amount` - Amount per person. Defaults to 2.5 (kg) if None.
///
/// # Returns
///
/// `ZakatDetails` where `zakat_due` is the total monetary value.
pub fn calculate_fitrah(
    person_count: u32,
    price_per_unit: impl Into<Decimal>,
    unit_amount: Option<Decimal>,
) -> Result<ZakatDetails, ZakatError> {
    let price_per_unit = price_per_unit.into();
    if person_count == 0 {
        return Err(ZakatError::InvalidInput("Person count must be greater than 0".to_string()));
    }
    if price_per_unit <= Decimal::ZERO {
        return Err(ZakatError::InvalidInput("Price per unit must be positive".to_string()));
    }

    let amount_per_person = unit_amount.unwrap_or(dec!(2.5)); // Default 2.5kg
    let total_people_decimal: Decimal = person_count.into();
    
    // Total assets in this context is just the total quantity needed in units, but ZakatDetails
    // is financial, so we represent everything in currency.
    // Total Value = person * amount_per_person * price
    let total_value = total_people_decimal * amount_per_person * price_per_unit;

    // Zakat Fitrah is obligatory on every individual who has sustenance for the day of Eid,
    // regardless of wealth thresholds (Nisab) applicable to Zakat Maal.
    // Therefore, we set Nisab to 0 to reflect that it is payable if the person has the means.
    
    Ok(ZakatDetails {
        total_assets: total_value,
        deductible_liabilities: Decimal::ZERO,
        net_assets: total_value,
        nisab_threshold: Decimal::ZERO, // Fitrah is obligatory, no wealth nisab in the same sense as Maal
        is_payable: true,
        zakat_due: total_value,
        wealth_type: crate::types::WealthType::Fitrah,
        status_reason: None,
        label: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fitrah_basic() {
        let price = dec!(10.0); // 10 currency per kg
        let people = 4;
        // Default 2.5kg * 4 people * 10 = 100
        let result = calculate_fitrah(people, price, None).unwrap();
        assert_eq!(result.zakat_due, dec!(100.0));
        assert!(result.is_payable);
    }

    #[test]
    fn test_fitrah_custom_amount() {
        let price = dec!(2.0);
        let people = 1;
        let amount = dec!(3.5); // Using liters or different mazhab
        // 1 * 3.5 * 2 = 7
        let result = calculate_fitrah(people, price, Some(amount)).unwrap();
        assert_eq!(result.zakat_due, dec!(7.0));
    }
}
