//! # Hawl (Lunar Year) Tracker
//!
//! In Islamic Law (Fiqh), wealth must be held for one full lunar year (Hawl) 
//! before Zakat becomes obligatory.
//! The lunar year is approximately 354 days long.
//!
//! This module provides logic to track acquisition dates and determine if Hawl 
//! is satisfied relative to a calculation date.
//!
//! ## Fuzzy Date Support
//! Users often don't remember the exact acquisition date. This module supports:
//! - **Exact dates**: Standard `NaiveDate` for precise tracking.
//! - **Approximate dates**: Fuzzy dates like "Ramadan 1445" or "Unknown".
//! - **Safe caution**: Unknown dates default to Hawl being satisfied (to avoid sin of non-payment).

use chrono::{NaiveDate, Local, Datelike};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use icu_calendar::{Date, islamic::IslamicCivil};

// =============================================================================
// Fuzzy Date Types (Feature 1: Smart Approximate Inputs)
// =============================================================================

/// Represents an approximate (fuzzy) date when exact date is unknown.
///
/// This allows users to specify dates based on Islamic months or indicate
/// that the date is completely unknown.
///
/// # Fiqh Principle
/// When the exact date is unknown, we apply "safe caution" (Ihtiyat):
/// - If truly unknown, assume Hawl is satisfied to avoid potential sin of non-payment.
/// - If a month is known (e.g., Ramadan), use the first day of that month.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum FuzzyDate {
    /// Wealth was acquired during Ramadan of the specified Hijri year.
    /// Uses the 1st of Ramadan for calculation.
    Ramadan(i32),
    /// Wealth was acquired during Muharram of the specified Hijri year.
    /// Uses the 1st of Muharram for calculation.
    Muharram(i32),
    /// Wealth was acquired during Shawwal of the specified Hijri year.
    /// Uses the 1st of Shawwal for calculation.
    Shawwal(i32),
    /// Wealth was acquired during Dhul Hijjah of the specified Hijri year.
    /// Uses the 1st of Dhul Hijjah for calculation.
    DhulHijjah(i32),
    /// Date is completely unknown.
    /// Safe caution: Hawl is assumed to be satisfied.
    Unknown,
}

impl FuzzyDate {
    /// Converts the fuzzy date to a Gregorian `NaiveDate`.
    ///
    /// For known months, returns the first day of that month.
    /// For `Unknown`, returns `None` (caller should apply safe caution).
    pub fn to_gregorian(&self) -> Option<NaiveDate> {
        match self {
            FuzzyDate::Ramadan(year) => Self::hijri_month_to_gregorian(*year, 9),
            FuzzyDate::Muharram(year) => Self::hijri_month_to_gregorian(*year, 1),
            FuzzyDate::Shawwal(year) => Self::hijri_month_to_gregorian(*year, 10),
            FuzzyDate::DhulHijjah(year) => Self::hijri_month_to_gregorian(*year, 12),
            FuzzyDate::Unknown => None,
        }
    }

    /// Converts a Hijri year and month to the corresponding Gregorian date.
    /// Returns the first day of the specified Hijri month.
    fn hijri_month_to_gregorian(hijri_year: i32, hijri_month: u8) -> Option<NaiveDate> {
        // Create a Hijri date for the 1st of the specified month using ICU
        let cal = IslamicCivil::new();
        let hijri_date = Date::try_new_islamic_civil_date_with_calendar(
            hijri_year, 
            hijri_month, 
            1,
            cal
        ).ok()?;
        
        // Convert to ISO (Gregorian)
        let iso_date = hijri_date.to_iso();
        
        // Convert ICU Date to chrono NaiveDate
        NaiveDate::from_ymd_opt(
            iso_date.year().number,
            iso_date.month().ordinal as u32,
            iso_date.day_of_month().0 as u32,
        )
    }

    /// Creates a fuzzy date for Ramadan of the specified Hijri year.
    pub fn ramadan(year: i32) -> Self {
        FuzzyDate::Ramadan(year)
    }

    /// Creates a fuzzy date for Muharram of the specified Hijri year.
    pub fn muharram(year: i32) -> Self {
        FuzzyDate::Muharram(year)
    }

    /// Creates a fuzzy date indicating the date is unknown.
    pub fn unknown() -> Self {
        FuzzyDate::Unknown
    }
}

/// Represents when wealth was acquired, supporting both exact and fuzzy dates.
///
/// This enum allows flexible date input while maintaining Fiqh compliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum AcquisitionDate {
    /// Exact date is known.
    Exact(NaiveDate),
    /// Approximate date based on Islamic calendar or unknown.
    Approximate(FuzzyDate),
}

impl AcquisitionDate {
    /// Creates an acquisition date from an exact Gregorian date.
    pub fn exact(date: NaiveDate) -> Self {
        AcquisitionDate::Exact(date)
    }

    /// Creates an acquisition date from a fuzzy date.
    pub fn approximate(fuzzy: FuzzyDate) -> Self {
        AcquisitionDate::Approximate(fuzzy)
    }

    /// Creates an acquisition date for Ramadan of the specified Hijri year.
    pub fn ramadan(year: i32) -> Self {
        AcquisitionDate::Approximate(FuzzyDate::Ramadan(year))
    }

    /// Creates an acquisition date for Muharram of the specified Hijri year.
    pub fn muharram(year: i32) -> Self {
        AcquisitionDate::Approximate(FuzzyDate::Muharram(year))
    }

    /// Creates an acquisition date indicating the date is unknown.
    /// Safe caution: Hawl will be assumed satisfied.
    pub fn unknown() -> Self {
        AcquisitionDate::Approximate(FuzzyDate::Unknown)
    }

    /// Resolves the acquisition date to a concrete `NaiveDate` if possible.
    /// Returns `None` for `Unknown` dates (safe caution applies).
    pub fn to_gregorian(&self) -> Option<NaiveDate> {
        match self {
            AcquisitionDate::Exact(date) => Some(*date),
            AcquisitionDate::Approximate(fuzzy) => fuzzy.to_gregorian(),
        }
    }

    /// Returns true if the date is unknown (safe caution applies).
    pub fn is_unknown(&self) -> bool {
        matches!(self, AcquisitionDate::Approximate(FuzzyDate::Unknown))
    }
}

impl From<NaiveDate> for AcquisitionDate {
    fn from(date: NaiveDate) -> Self {
        AcquisitionDate::Exact(date)
    }
}

impl From<FuzzyDate> for AcquisitionDate {
    fn from(fuzzy: FuzzyDate) -> Self {
        AcquisitionDate::Approximate(fuzzy)
    }
}

// =============================================================================
// Hawl Tracker
// =============================================================================

/// Tracks the holding period of an asset to determine Zakat eligibility (Hawl).
///
/// ## Features
/// - Supports exact and fuzzy (approximate) acquisition dates.
/// - Uses ICU calendar for precise Hijri calculations.
/// - Applies "safe caution" (Ihtiyat) for unknown dates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HawlTracker {
    /// The date the asset was acquired or reached Nisab.
    /// Supports both exact dates and fuzzy dates (Ramadan, Muharram, Unknown).
    #[serde(default)]
    pub acquisition_date: Option<AcquisitionDate>,
    /// Legacy field for backward compatibility.
    /// Deprecated: Use `acquisition_date` instead.
    #[serde(skip_serializing, default)]
    #[deprecated(since = "1.2.0", note = "Use `acquisition_date` with `AcquisitionDate::Exact` instead")]
    #[allow(dead_code)] // Kept for deserialization compatibility
    legacy_date: Option<NaiveDate>,
    /// The date Zakat is being calculated for (defaults to today).
    pub calculation_date: NaiveDate,
}

impl HawlTracker {
    /// Creates a new Hawl tracker with a specific calculation date.
    #[allow(deprecated)]
    pub fn new(calculation_date: NaiveDate) -> Self {
        Self {
            acquisition_date: None,
            legacy_date: None,
            calculation_date,
        }
    }

    /// Sets the acquisition date using an exact `NaiveDate`.
    /// This is the traditional API for backward compatibility.
    pub fn acquired_on(mut self, date: NaiveDate) -> Self {
        self.acquisition_date = Some(AcquisitionDate::Exact(date));
        self
    }

    /// Sets the acquisition date using a fuzzy date (e.g., Ramadan 1445).
    pub fn acquired_approximately(mut self, fuzzy: FuzzyDate) -> Self {
        self.acquisition_date = Some(AcquisitionDate::Approximate(fuzzy));
        self
    }

    /// Sets the acquisition date using the `AcquisitionDate` enum.
    pub fn with_acquisition_date(mut self, date: AcquisitionDate) -> Self {
        self.acquisition_date = Some(date);
        self
    }

    /// Marks the acquisition date as unknown (safe caution: Hawl assumed satisfied).
    pub fn date_unknown(mut self) -> Self {
        self.acquisition_date = Some(AcquisitionDate::unknown());
        self
    }

    /// Checks if the Hawl (1 Lunar Year) has been satisfied.
    ///
    /// Uses `icu_calendar` for precise Hijri conversion.
    ///
    /// # Returns
    /// - `true` if:
    ///   - `acquisition_date` is set AND >= 1 Hijri year has passed, OR
    ///   - `acquisition_date` is `Unknown` (safe caution applies).
    /// - `false` otherwise.
    pub fn is_satisfied(&self) -> bool {
        match &self.acquisition_date {
            Some(acq_date) => {
                // Handle Unknown dates with safe caution (assume Hawl satisfied)
                if acq_date.is_unknown() {
                    return true;
                }

                // Try to resolve to a concrete date
                match acq_date.to_gregorian() {
                    Some(start_date) => {
                        // Try precise calculation first
                        if let Ok(satisfied) = self.is_satisfied_precise(start_date) {
                            satisfied
                        } else {
                            // Fallback to approximation if conversion fails
                            self.days_elapsed(start_date) >= 354
                        }
                    }
                    None => {
                        // If we can't resolve the date, apply safe caution
                        true
                    }
                }
            }
            None => false,
        }
    }

    /// Returns the reason for Hawl satisfaction status.
    /// Useful for audit logs and user explanations.
    pub fn satisfaction_reason(&self) -> &'static str {
        match &self.acquisition_date {
            Some(acq_date) => {
                if acq_date.is_unknown() {
                    "Date unknown - safe caution applied (Hawl assumed satisfied)"
                } else if self.is_satisfied() {
                    "One lunar year has passed since acquisition"
                } else {
                    "Less than one lunar year since acquisition"
                }
            }
            None => "No acquisition date set",
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
        
        match &self.acquisition_date {
            Some(acq_date) => {
                // Unknown dates are considered 100% complete (safe caution)
                if acq_date.is_unknown() {
                    return Decimal::ONE;
                }
                
                match acq_date.to_gregorian() {
                    Some(start) => {
                        let days = self.days_elapsed(start);
                        if days <= 0 {
                            Decimal::ZERO
                        } else {
                            let d = Decimal::from_i64(days).unwrap_or(Decimal::ZERO);
                            let hawl = Decimal::from(354);
                            d / hawl
                        }
                    }
                    None => Decimal::ONE, // Can't resolve, apply safe caution
                }
            }
            None => Decimal::ZERO,
        }
    }
}

impl Default for HawlTracker {
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            acquisition_date: None,
            legacy_date: None,
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

    // =============================================================================
    // Feature 1 Tests: Fuzzy Dates
    // =============================================================================

    #[test]
    fn test_fuzzy_date_unknown_applies_safe_caution() {
        let today = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let tracker = HawlTracker::new(today).date_unknown();
        
        // Unknown date should always satisfy Hawl (safe caution)
        assert!(tracker.is_satisfied(), "Unknown date should assume Hawl is satisfied");
        assert_eq!(tracker.completion_percentage(), Decimal::ONE);
        assert!(tracker.satisfaction_reason().contains("safe caution"));
    }

    #[test]
    fn test_fuzzy_date_ramadan_conversion() {
        // Ramadan 1445 started around March 10-11, 2024
        let fuzzy = FuzzyDate::Ramadan(1445);
        let gregorian = fuzzy.to_gregorian();
        
        assert!(gregorian.is_some(), "Ramadan 1445 should convert to Gregorian");
        let date = gregorian.unwrap();
        
        // Should be in March 2024
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 3);
    }

    #[test]
    fn test_fuzzy_date_muharram_conversion() {
        // Muharram 1446 started around July 7-8, 2024
        let fuzzy = FuzzyDate::Muharram(1446);
        let gregorian = fuzzy.to_gregorian();
        
        assert!(gregorian.is_some(), "Muharram 1446 should convert to Gregorian");
        let date = gregorian.unwrap();
        
        // Should be in July 2024
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 7);
    }

    #[test]
    fn test_hawl_with_approximate_ramadan_date() {
        // If acquired in Ramadan 1445 (March 2024)
        // and calculating in March 2025 (after 1 Hijri year)
        let calc_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let tracker = HawlTracker::new(calc_date)
            .acquired_approximately(FuzzyDate::Ramadan(1445));
        
        // Should be satisfied (1 full Hijri year has passed)
        assert!(tracker.is_satisfied());
    }

    #[test]
    fn test_hawl_with_approximate_ramadan_date_not_satisfied() {
        // If acquired in Ramadan 1446 (Feb-March 2025)
        // and calculating in same period
        let calc_date = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let tracker = HawlTracker::new(calc_date)
            .acquired_approximately(FuzzyDate::Ramadan(1446));
        
        // Should NOT be satisfied (same year)
        assert!(!tracker.is_satisfied());
    }

    #[test]
    fn test_acquisition_date_from_naive_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let acq: AcquisitionDate = date.into();
        
        assert!(matches!(acq, AcquisitionDate::Exact(_)));
        assert_eq!(acq.to_gregorian(), Some(date));
    }

    #[test]
    fn test_acquisition_date_unknown() {
        let acq = AcquisitionDate::unknown();
        
        assert!(acq.is_unknown());
        assert!(acq.to_gregorian().is_none());
    }
}
