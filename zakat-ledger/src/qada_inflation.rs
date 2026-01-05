//! # Retrospective Qada (Missed Zakat) Calculator
//!
//! Handles the calculation of missed Zakat payments (Qada) adjusting for inflation
//! using the **Gold Standard Method**.
//!
//! ## Fiqh Principle
//! Paying missed Zakat from years ago using the same currency face value is unjust
//! if the currency has devalued significantly.
//!
//! **Method**:
//! 1. Calculate how many *grams of gold* the original Zakat amount could buy *at the time it was due*.
//! 2. The debt is recorded in "Gold Grams".
//! 3. Pay the current value of those "Gold Grams" today.
//!
//! *Reference*: Fatwas on Inflationary Debt repayment suggesting Gold as a stable store of value for long-term debts.

use rust_decimal::Decimal;
use chrono::NaiveDate;
use serde::{Serialize, Deserialize};

/// Trait to provide historical data for inflation adjustment.
pub trait InflationIndexProvider {
    /// Get the gold price per gram at a specific date.
    fn get_gold_price_at(&self, date: NaiveDate) -> Option<Decimal>;
    
    /// Get the CPI (Consumer Price Index) at a specific date (optional alternative).
    fn get_cpi_at(&self, date: NaiveDate) -> Option<Decimal>;
}

/// A calculator for missing Zakat payments.
pub struct MissedZakatCalculator<'a, P: InflationIndexProvider> {
    provider: &'a P,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InflationAdjustmentResult {
    pub original_amount: Decimal,
    pub due_date: NaiveDate,
    pub payment_date: NaiveDate,
    pub gold_price_at_due: Decimal,
    pub gold_price_at_payment: Decimal,
    pub grams_owed: Decimal,
    pub adjusted_amount_to_pay: Decimal,
    pub inflation_delta: Decimal, // Extra amount due to inflation
}

impl<'a, P: InflationIndexProvider> MissedZakatCalculator<'a, P> {
    pub fn new(provider: &'a P) -> Self {
        Self { provider }
    }

    /// Calculates the inflation-adjusted amount using the Gold Standard method.
    /// 
    /// Returns `None` if historical price data is missing.
    pub fn calculate_gold_standard(
        &self,
        original_amount: Decimal,
        due_date: NaiveDate,
        payment_date: NaiveDate,
    ) -> Option<InflationAdjustmentResult> {
        let price_at_due = self.provider.get_gold_price_at(due_date)?;
        let price_at_payment = self.provider.get_gold_price_at(payment_date)?;

        if price_at_due.is_zero() {
            return None; 
        }

        // 1. Convert debt to Gold Grams
        let grams_owed = original_amount / price_at_due;

        // 2. Convert back to Currency at Payment Date
        let adjusted_amount = grams_owed * price_at_payment;
        
        let delta = adjusted_amount - original_amount;

        Some(InflationAdjustmentResult {
            original_amount,
            due_date,
            payment_date,
            gold_price_at_due: price_at_due,
            gold_price_at_payment: price_at_payment,
            grams_owed,
            adjusted_amount_to_pay: adjusted_amount,
            inflation_delta: delta,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use chrono::Datelike;

    struct MockProvider;
    impl InflationIndexProvider for MockProvider {
        fn get_gold_price_at(&self, date: NaiveDate) -> Option<Decimal> {
            // Mock data: 
            // 2010: $40/gram
            // 2025: $80/gram
            if date.year() == 2010 {
                Some(dec!(40))
            } else if date.year() == 2025 {
                Some(dec!(80))
            } else {
                None
            }
        }
        fn get_cpi_at(&self, _date: NaiveDate) -> Option<Decimal> { None }
    }

    #[test]
    fn test_inflation_adjustment() {
        let provider = MockProvider;
        let calculator = MissedZakatCalculator::new(&provider);

        let due_date = NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
        let payment_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        
        // Owed $1000 in 2010.
        // Gold was $40/g. So owed 25 grams.
        // Now gold is $80/g. Should pay 25 * 80 = $2000.
        
        let result = calculator.calculate_gold_standard(dec!(1000), due_date, payment_date).unwrap();
        
        assert_eq!(result.grams_owed, dec!(25));
        assert_eq!(result.adjusted_amount_to_pay, dec!(2000));
        assert_eq!(result.inflation_delta, dec!(1000));
    }
}
