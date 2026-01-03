//! # Hawl (Lunar Year) Tracker
//!
//! In Islamic Law (Fiqh), wealth must be held for one full lunar year (Hawl) 
//! before Zakat becomes obligatory.
//! The lunar year is approximately 354 days long.
//!
//! This module provides logic to track acquisition dates and determine if Hawl 
//! is satisfied relative to a calculation date.

use chrono::{NaiveDate, Local};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Tracks the holding period of an asset to determine Zakat eligibility (Hawl).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HawlTracker {
    /// The date the asset was acquired or reached Nisab.
    pub acquisition_date: Option<NaiveDate>,
    /// The date Zakat is being calculated for (defaults to today).
    pub calculation_date: NaiveDate,
}

impl HawlTracker {
    /// Creates a new Hawl tracker with a specific calculation date.
    pub fn new(calculation_date: NaiveDate) -> Self {
        Self {
            acquisition_date: None,
            calculation_date,
        }
    }

    /// Sets the acquisition date.
    pub fn acquired_on(mut self, date: NaiveDate) -> Self {
        self.acquisition_date = Some(date);
        self
    }

    /// Checks if the Hawl (354 days) has been satisfied.
    ///
    /// # Returns
    /// - `true` if `acquisition_date` is set AND >= 354 days have passed.
    /// - `false` otherwise.
    pub fn is_satisfied(&self) -> bool {
        match self.acquisition_date {
            Some(start_date) => self.days_elapsed(start_date) >= 354,
            None => false,
        }
    }

    /// Returns the number of days elapsed between acquisition and calculation.
    pub fn days_elapsed(&self, start_date: NaiveDate) -> i64 {
        (self.calculation_date - start_date).num_days()
    }

    /// Returns the percentage of the Hawl completed (0.0 to 1.0+).
    /// Useful for pro-rata calculations if needed (though Zakat is usually binary).
    pub fn completion_percentage(&self) -> Decimal {
        use rust_decimal::prelude::FromPrimitive;
        
        match self.acquisition_date {
            Some(start) => {
                let days = self.days_elapsed(start);
                if days <= 0 {
                    Decimal::ZERO
                } else {
                    let d = Decimal::from_i64(days).unwrap_or(Decimal::ZERO);
                    let hawl = Decimal::from(354);
                    d / hawl
                }
            },
            None => Decimal::ZERO,
        }
    }
}

impl Default for HawlTracker {
    fn default() -> Self {
        Self {
            acquisition_date: None,
            calculation_date: Local::now().date_naive(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_hawl_satisfaction() {
        let today = NaiveDate::from_ymd_opt(2023, 10, 1).unwrap();
        let tracker = HawlTracker::new(today);

        // Case 1: Acquired exactly 354 days ago -> Satisfied
        let date_valid = today - Duration::days(354);
        let t1 = tracker.clone().acquired_on(date_valid);
        assert!(t1.is_satisfied());

        // Case 2: Acquired 353 days ago -> Not Satisfied
        let date_invalid = today - Duration::days(353);
        let t2 = tracker.clone().acquired_on(date_invalid);
        assert!(!t2.is_satisfied());

        // Case 3: Acquired 400 days ago -> Satisfied
        let date_old = today - Duration::days(400);
        let t3 = tracker.clone().acquired_on(date_old);
        assert!(t3.is_satisfied());
    }

    #[test]
    fn test_default_behavior() {
        let tracker = HawlTracker::default();
        assert!(!tracker.is_satisfied()); // No acquisition date
    }
}
