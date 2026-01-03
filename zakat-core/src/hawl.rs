//! # Hawl (Lunar Year) Tracker
//!
//! In Islamic Law (Fiqh), wealth must be held for one full lunar year (Hawl) 
//! before Zakat becomes obligatory.
//! The lunar year is approximately 354 days long.
//!
//! This module provides logic to track acquisition dates and determine if Hawl 
//! is satisfied relative to a calculation date.

use chrono::{NaiveDate, Local, Datelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use icu_calendar::{Date, islamic::IslamicCivil};

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

    /// Checks if the Hawl (1 Lunar Year) has been satisfied.
    ///
    /// Uses `icu_calendar` for precise Hijri conversion.
    ///
    /// # Returns
    /// - `true` if `acquisition_date` is set AND >= 1 Hijri year has passed.
    /// - `false` otherwise.
    pub fn is_satisfied(&self) -> bool {
        match self.acquisition_date {
            Some(start_date) => {
                // Try precise calculation first
                if let Ok(satisfied) = self.is_satisfied_precise(start_date) {
                    satisfied
                } else {
                    // Fallback to approximation if conversion fails
                    self.days_elapsed(start_date) >= 354
                }
            },
            None => false,
        }
    }

    fn is_satisfied_precise(&self, start: NaiveDate) -> Result<bool, &'static str> {
        let now = self.calculation_date;

        // 1. Convert Chrono NaiveDate to ICU Date<Iso>
        let start_iso = Date::try_new_iso_date(start.year(), start.month() as u8, start.day() as u8)
            .map_err(|_| "Invalid start date")?;
        let now_iso = Date::try_new_iso_date(now.year(), now.month() as u8, now.day() as u8)
            .map_err(|_| "Invalid calculation date")?;

        // 2. Convert to Hijri (Islamic Civil)
        let cal = IslamicCivil::new();
        let start_hijri = start_iso.to_calendar(cal.clone());
        let now_hijri = now_iso.to_calendar(cal);

        // 3. Compare dates
        let passed_years = now_hijri.year().number - start_hijri.year().number;
        
        if passed_years > 1 {
            return Ok(true);
        }
        if passed_years == 1 {
            // Compare month and day
            if now_hijri.month().ordinal > start_hijri.month().ordinal {
                return Ok(true);
            }
            if now_hijri.month().ordinal == start_hijri.month().ordinal 
               && now_hijri.day_of_month().0 >= start_hijri.day_of_month().0 {
                return Ok(true);
            }
        }
        
        Ok(false)
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

    #[test]
    fn test_precise_hijri_hawl() {
        // 1 Ramadan 1444 is approx March 23, 2023
        // 1 Ramadan 1445 is approx March 11, 2024
        
        let start = NaiveDate::from_ymd_opt(2023, 3, 23).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 11).unwrap();
        
        let tracker = HawlTracker::new(end).acquired_on(start);
        assert!(tracker.is_satisfied(), "Should be satisfied exactly on 1 Ramadan 1445");
        
        let day_before = end.pred_opt().unwrap();
        let tracker_early = HawlTracker::new(day_before).acquired_on(start);
        assert!(!tracker_early.is_satisfied(), "Should NOT be satisfied one day before 1 Ramadan 1445");
    }
}
