//! Ledger Asset
//!
//! A Zakat-calculable asset backed by ledger events.

use crate::events::LedgerEvent;
use crate::pricing::InMemoryPriceHistory;
use crate::timeline::simulate_timeline;
use crate::analyzer::analyze_hawl;
use zakat_core::types::{ZakatDetails, WealthType, ZakatError, CalculationStep};
use zakat_core::traits::{CalculateZakat, ZakatConfigArgument};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// An asset whose balance and Hawl status are derived from a ledger of events.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LedgerAsset {
    pub id: Uuid,
    pub label: String,
    pub events: Vec<LedgerEvent>,
    pub prices: InMemoryPriceHistory, 
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl LedgerAsset {
    pub fn new(label: impl Into<String>, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            events: Vec::new(),
            prices: InMemoryPriceHistory::new(),
            start_date,
            end_date,
        }
    }

    pub fn with_event(mut self, event: LedgerEvent) -> Self {
        self.events.push(event);
        self
    }

    pub fn with_price(mut self, date: NaiveDate, price: Decimal) -> Self {
        self.prices.add_price(date, price);
        self
    }
}

impl CalculateZakat for LedgerAsset {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, _config: C) -> Result<ZakatDetails, ZakatError> {
        // Run simulation
        let timeline = simulate_timeline(self.events.clone(), &self.prices, self.start_date, self.end_date, None)?;
        
        // Run analyzer
        let result = analyze_hawl(&timeline);
        
        // Determine Nisab from last day
        let final_nisab = timeline.last().map(|d| d.nisab_threshold).unwrap_or(Decimal::ZERO);
        
        // Assume Business/Monetary for aggregation
        let wealth_type = WealthType::Business;
        
        // Build Trace
        let mut final_trace = Vec::new();
        final_trace.push(CalculationStep::initial("step-ledger-balance", "Ledger Closing Balance", result.total_balance));
        final_trace.push(CalculationStep::compare("step-nisab-check", "Nisab Threshold (End Date)", final_nisab));
        
        if result.is_due {
             final_trace.push(CalculationStep::info("info-hawl-met", format!("Hawl Met: {} days held since {}", result.current_streak_days, result.hawl_start_date.map(|d| d.to_string()).unwrap_or_default())));
             final_trace.push(CalculationStep::rate("step-rate", "Zakat Rate", dec!(0.025)));
             final_trace.push(CalculationStep::result("step-due", "Zakat Due", result.zakat_due));
        } else {
             if let Some(breach) = result.last_breach {
                 final_trace.push(CalculationStep::info("info-hawl-broken", format!("Hawl reset due to breach on {}", breach)));
             } else {
                 final_trace.push(CalculationStep::info("info-hawl-short", "Wealth below Nisab or period too short"));
             }
             final_trace.push(CalculationStep::info("info-hawl-progress", format!("Current Streak: {}/354 days", result.current_streak_days)));
        }
        
        let mut detailed_details = ZakatDetails::with_breakdown(
            result.total_balance,
            Decimal::ZERO,
            final_nisab,
            dec!(0.025),
            wealth_type.clone(),
            final_trace
        ).with_label(self.label.clone());
        
        // Force the payable status from analyzer results
        detailed_details.is_payable = result.is_due && result.total_balance >= final_nisab;
        detailed_details.zakat_due = if detailed_details.is_payable { result.zakat_due } else { Decimal::ZERO };
        if !detailed_details.is_payable {
            detailed_details.status_reason = Some(format!("Hawl not met: {}/354 days", result.current_streak_days));
        }

        Ok(detailed_details)
    }

    fn get_label(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}
