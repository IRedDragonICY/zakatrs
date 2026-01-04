//! Hawl Analyzer
//!
//! Analyzes a timeline to determine Zakat eligibility based on Hawl rules.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use super::timeline::DailyBalance;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Result of Hawl analysis on a ledger timeline.
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

impl LedgerZakatResult {
    /// Returns a human-readable breakdown of the Hawl analysis.
    ///
    /// Useful for debugging, audit trails, and user-facing explanations.
    pub fn explain(&self) -> String {
        let mut lines = Vec::new();
        
        lines.push("=== Ledger Zakat Analysis ===".to_string());
        lines.push(format!("Current Balance: {}", self.total_balance));
        lines.push(format!("Zakat Due: {} ({})", 
            self.zakat_due,
            if self.is_due { "PAYABLE" } else { "NOT YET DUE" }
        ));
        lines.push(String::new());
        lines.push("--- Hawl Status ---".to_string());
        lines.push(format!("Days Above Nisab: {}/354 ({:.1}%)", 
            self.current_streak_days,
            self.completion_percentage * dec!(100)
        ));
        
        if let Some(start) = self.hawl_start_date {
            lines.push(format!("Hawl Started: {}", start));
        }
        
        if let Some(breach) = self.last_breach {
            lines.push(format!("Last Nisab Breach: {}", breach));
        }
        
        lines.join("\n")
    }
}

/// Analyzes a timeline to determine if Hawl is satisfied and calculate Zakat.
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
            info!(
                date = %day.date, 
                balance = %day.balance, 
                "Hawl breach detected - balance fell below Nisab"
            );
            last_breach = Some(day.date);
            break; 
        }
        streak_start_date = day.date;
    }
    
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

