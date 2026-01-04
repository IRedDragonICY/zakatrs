//! Historical Price Provider
//!
//! Provides price data for Nisab calculations over time.

use rust_decimal::Decimal;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

use zakat_core::types::{ZakatError, ErrorDetails};

/// Trait for providing historical Nisab prices.
pub trait HistoricalPriceProvider {
    fn get_nisab_threshold(&self, date: NaiveDate) -> Result<Decimal, ZakatError>;

    /// Returns the date of the next price update after the given date.
    /// Returns None if there are no known future price changes (implies constant price indefinitely).
    /// Returns Some(date) if the price changes on `date`.
    fn next_price_change(&self, after: NaiveDate) -> Option<NaiveDate> {
        // Default implementation: assumes we don't know, so we can't optimize skipping.
        // Returning Some(after + 1) would force daily checks, which is safe.
        Some(after + chrono::Duration::days(1))
    }
}

/// In-memory historical price provider.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct InMemoryPriceHistory {
    prices: BTreeMap<NaiveDate, Decimal>,
}

impl Default for InMemoryPriceHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPriceHistory {
    pub fn new() -> Self {
        Self {
            prices: BTreeMap::new(),
        }
    }

    pub fn add_price(&mut self, date: NaiveDate, price: Decimal) {
        self.prices.insert(date, price);
    }
}

impl HistoricalPriceProvider for InMemoryPriceHistory {
    fn get_nisab_threshold(&self, date: NaiveDate) -> Result<Decimal, ZakatError> {
         // Return the most recent price before or on that date.
         self.prices.range(..=date).next_back().map(|(_, &price)| price)
            .ok_or_else(|| ZakatError::ConfigurationError(Box::new(ErrorDetails { 
                code: zakat_core::types::ZakatErrorCode::ConfigMissing,
                reason_key: "error-nisab-price-missing".to_string(),
                args: Some(std::collections::HashMap::from([("date".to_string(), date.to_string())])),
                source_label: Some("HistoricalPriceProvider".to_string()),
                suggestion: Some("Ensure historical prices are loaded for the requested date.".to_string()),
                ..Default::default()
            })))
    }

    fn next_price_change(&self, after: NaiveDate) -> Option<NaiveDate> {
        use std::ops::Bound;
        // Find the first key strictly greater than 'after'
        self.prices.range((Bound::Excluded(after), Bound::Unbounded))
            .next()
            .map(|(date, _)| *date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use chrono::NaiveDate;

    #[test]
    fn test_exact_date_lookup() {
        let mut history = InMemoryPriceHistory::new();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        history.add_price(date, dec!(8500));
        
        let result = history.get_nisab_threshold(date).unwrap();
        assert_eq!(result, dec!(8500));
    }
    
    #[test]
    fn test_interpolation_carry_forward() {
        // Feature 3: Ledger should use last known price if exact date not found
        let mut history = InMemoryPriceHistory::new();
        
        // Add price for Jan 1st
        let jan1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        history.add_price(jan1, dec!(8000));
        
        // Add price for Feb 1st
        let feb1 = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
        history.add_price(feb1, dec!(8500));
        
        // Query Jan 15 - should use Jan 1 price (carry forward)
        let jan15 = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let result = history.get_nisab_threshold(jan15).unwrap();
        assert_eq!(result, dec!(8000)); // Uses Jan 1 price
        
        // Query Feb 15 - should use Feb 1 price
        let feb15 = NaiveDate::from_ymd_opt(2025, 2, 15).unwrap();
        let result = history.get_nisab_threshold(feb15).unwrap();
        assert_eq!(result, dec!(8500)); // Uses Feb 1 price
    }
    
    #[test]
    fn test_no_price_before_date_returns_error() {
        let mut history = InMemoryPriceHistory::new();
        
        // Add price for Feb 1st only
        let feb1 = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
        history.add_price(feb1, dec!(8500));
        
        // Query Jan 15 - no price exists before this date
        let jan15 = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let result = history.get_nisab_threshold(jan15);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_next_price_change() {
        let mut history = InMemoryPriceHistory::new();
        
        let jan1 = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let feb1 = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
        let mar1 = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        
        history.add_price(jan1, dec!(8000));
        history.add_price(feb1, dec!(8500));
        history.add_price(mar1, dec!(9000));
        
        // After Jan 1, next change is Feb 1
        assert_eq!(history.next_price_change(jan1), Some(feb1));
        
        // After Feb 1, next change is Mar 1
        assert_eq!(history.next_price_change(feb1), Some(mar1));
        
        // After Mar 1, no more changes
        assert_eq!(history.next_price_change(mar1), None);
    }
}
