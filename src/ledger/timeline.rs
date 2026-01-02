use super::events::LedgerEvent;
use super::pricing::HistoricalPriceProvider;
use chrono::{NaiveDate, Duration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DailyBalance {
    pub date: NaiveDate,
    pub balance: Decimal,
    pub nisab_threshold: Decimal,
    pub is_above_nisab: bool,
}

use crate::types::{ZakatError, InvalidInputDetails};

pub fn simulate_timeline<P: HistoricalPriceProvider>(
    events: Vec<LedgerEvent>,
    price_provider: &P,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<DailyBalance>, ZakatError> {
    if start_date > end_date {
        return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
            field: "date_range".to_string(), 
            value: format!("{} > {}", start_date, end_date), 
            reason_key: "error-date-range-invalid".to_string(),
            args: None,
            source_label: Some("simulate_timeline".to_string()), 
            asset_id: None 
        })));
    }

    let mut timeline = Vec::new();
    let mut current_balance = Decimal::ZERO;
    
    // Ensure events are sorted by date
    let mut sorted_events = events;
    sorted_events.sort_by_key(|e| e.date);
    
    let mut current_date = start_date;
    
    // We need an iterator for events
    let mut event_iter = sorted_events.into_iter().peekable();
    
    while current_date <= end_date {
        // Process all events for the current day
        while let Some(event) = event_iter.peek() {
            if event.date == current_date {
                if event.amount < Decimal::ZERO {
                     return Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                        field: "amount".to_string(),
                        value: event.amount.to_string(),
                        reason_key: "error-amount-positive".to_string(),
                        args: None,
                        source_label: Some("simulate_timeline".to_string()),
                        asset_id: Some(event.id),
                    })));
                }

                use super::events::TransactionType::*;
                match event.transaction_type {
                    Deposit | Income | Profit => current_balance += event.amount,
                    Withdrawal | Expense | Loss => current_balance -= event.amount,
                }
                
                event_iter.next();
            } else if event.date < current_date {
                // Should not happen if sorted and logic is correct, but safe to consume
                event_iter.next(); 
            } else {
                // Event is in future relative to current_date
                break;
            }
        }
        
        // Fail if price provider fails
        let nisab_threshold = price_provider.get_nisab_threshold(current_date)?;
        
        timeline.push(DailyBalance {
            date: current_date,
            balance: current_balance,
            nisab_threshold,
            is_above_nisab: current_balance >= nisab_threshold,
        });
        
        current_date += Duration::days(1);
    }
    
    Ok(timeline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::events::TransactionType;
    use super::super::pricing::InMemoryPriceHistory;
    use rust_decimal_macros::dec;
    
    #[test]
    fn test_the_dip() {
        // Scenario:
        // Start: Jan 1, Balance 10,000. Nisab 1,000.
        // Dip: June 1, Withdraw 9,600 -> Balance 400. (Below Nisab)
        // Recovery: June 5, Deposit 9,600 -> Balance 10,000.
        // End: Dec 31.
        
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let dip_date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
        let recovery_date = NaiveDate::from_ymd_opt(2023, 6, 5).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        
        use crate::types::WealthType;
        let events = vec![
            LedgerEvent::new(start_date, dec!(10000), WealthType::Business, TransactionType::Deposit, Some("Initial".to_string())),
            LedgerEvent::new(dip_date, dec!(9600), WealthType::Business, TransactionType::Withdrawal, Some("Big Expense".to_string())),
            LedgerEvent::new(recovery_date, dec!(9600), WealthType::Business, TransactionType::Deposit, Some("Recovery".to_string())),
        ];
        
        let mut prices = InMemoryPriceHistory::new();
        // Set constant Nisab for simplicity
        prices.add_price(start_date, dec!(1000)); 
        
        let timeline = simulate_timeline(events, &prices, start_date, end_date).expect("Simulation failed");
        
        // Check finding specific days
        let day_jan_1 = timeline.iter().find(|d| d.date == start_date).unwrap();
        assert!(day_jan_1.is_above_nisab);
        assert_eq!(day_jan_1.balance, dec!(10000));

        let day_dip = timeline.iter().find(|d| d.date == dip_date).unwrap();
        assert!(!day_dip.is_above_nisab, "Should be below Nisab on dip date");
        assert_eq!(day_dip.balance, dec!(400));
        
        let day_recovery = timeline.iter().find(|d| d.date == recovery_date).unwrap();
        assert!(day_recovery.is_above_nisab, "Should be back above Nisab");
        assert_eq!(day_recovery.balance, dec!(10000));
        
        // Count days below nisab (June 1, 2, 3, 4) -> 4 days
        let days_below = timeline.iter().filter(|d| !d.is_above_nisab).count();
        assert_eq!(days_below, 4);
    }
}
