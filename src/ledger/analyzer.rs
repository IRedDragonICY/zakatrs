use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::timeline::DailyBalance;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerZakatResult {
    pub zakat_due: Decimal,
    pub is_due: bool,
    pub current_streak_days: i64,
    pub hawl_start_date: Option<NaiveDate>,
    pub completion_percentage: Decimal,
    pub total_balance: Decimal,
    pub last_breach: Option<NaiveDate>,
}

pub fn analyze_hawl(timeline: &[DailyBalance]) -> LedgerZakatResult {
    if timeline.is_empty() {
        return LedgerZakatResult {
            zakat_due: Decimal::ZERO,
            is_due: false,
            current_streak_days: 0,
            hawl_start_date: None,
            completion_percentage: Decimal::ZERO,
            total_balance: Decimal::ZERO,
            last_breach: None,
        };
    }

    let last_day = timeline.last().unwrap();
    let current_balance = last_day.balance;
    let today = last_day.date;

    // Iterate backwards to find the start of the current streak
    let mut streak_start_date = today;
    let mut last_breach = None;
    
    // Check if we are currently below nisab
    if !last_day.is_above_nisab {
        return LedgerZakatResult {
            zakat_due: Decimal::ZERO,
            is_due: false,
            current_streak_days: 0,
            hawl_start_date: None,
            completion_percentage: Decimal::ZERO,
            total_balance: current_balance,
            last_breach: Some(today),
        };
    }

    // Scan backwards from the *second to last* item
    for day in timeline.iter().rev().skip(1) {
        if !day.is_above_nisab {
            last_breach = Some(day.date);
            break; 
        }
        streak_start_date = day.date;
    }
    
    // If we scanned everything and never found a breach, streak_start_date is timeline[0].date
    // If we found a breach, streak_start_date is the day AFTER the breach (which our loop logic handles by setting streak_start_date = day.date only if above nisab)
    // Actually, careful: if day is NOT above nisab, loop breaks. The `streak_start_date` remains at the previous iteration's value (which was above nisab).
    
    let days_held = (today - streak_start_date).num_days() + 1; // inclusive count
    
    // Lunar year is approx 354 days
    let lunar_year_days = 354;
    let is_due = days_held >= lunar_year_days;
    
    let completion_percentage = if days_held >= 354 {
        Decimal::ONE
    } else {
        Decimal::from(days_held) / Decimal::from(lunar_year_days)
    };

    let zakat_due = if is_due {
        current_balance * dec!(0.025)
    } else {
        Decimal::ZERO
    };

    LedgerZakatResult {
        zakat_due,
        is_due,
        current_streak_days: days_held,
        hawl_start_date: Some(streak_start_date),
        completion_percentage,
        total_balance: current_balance,
        last_breach,
    }
}
