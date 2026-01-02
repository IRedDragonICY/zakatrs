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
    
    // Use an option to track the last known nisab to avoid fetching if not needed,
    // though for now we simplify to fetching or jumping.
    let mut current_nisab;

    while current_date <= end_date {
        let mut balance_changed = false;

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
                balance_changed = true;
                event_iter.next();
            } else if event.date < current_date {
                // Should not happen if sorted and logic is correct, but safe to consume
                event_iter.next(); 
            } else {
                // Event is in future relative to current_date
                break;
            }
        }
        
        // Update Nisab only if we suspect a change (or just fetch it if we jumped)
        // But since we track price changes via looking ahead, we can rely on cached value
        // UNLESS this is a "jump target" day where things might have changed.
        // Safest is to fetch on the days we explicitly visit loops, 
        // OR rely on the jump logic to essentially guarantee we stop ON key dates.
        
        // Here we just fetch it for correct current state
        if balance_changed {
             // If we just processed events, or if we just landed here, 
             // we need to make sure nisab is correct for "today".
             // Since we might have jumped, let's refresh.
            current_nisab = price_provider.get_nisab_threshold(current_date)?;
        } else {
            // Check if today IS a price change day?
            // Expensive to check 'is today a change?'.
            // Prefer: get standard value.
             current_nisab = price_provider.get_nisab_threshold(current_date)?;
        }
        
        // Push result for TODAY
        timeline.push(DailyBalance {
            date: current_date,
            balance: current_balance,
            nisab_threshold: current_nisab,
            is_above_nisab: current_balance >= current_nisab,
        });

        // TIME JUMP LOGIC
        // Determine the next interesting date
        let next_event_date = event_iter.peek().map(|e| e.date).unwrap_or(end_date + Duration::days(1));
        
        // Use optimized next_price_change lookup
        // If returns None, it means no more changes -> Infinity
        let next_price_date = price_provider.next_price_change(current_date)
            .unwrap_or(end_date + Duration::days(1));
            
        let jump_target = std::cmp::min(next_event_date, next_price_date);
        
        // We can fill days from (current_date + 1) up to min(jump_target, end_date + 1)
        // Strictly less than jump_target because on jump_target something distinct happens that we want to process in main loop.
        
        let fill_until = std::cmp::min(jump_target, end_date + Duration::days(1));
        
        // Ensure we don't go backwards
        if fill_until > current_date + Duration::days(1) {
            let days_to_fill = (fill_until - (current_date + Duration::days(1))).num_days();
            
            if days_to_fill > 0 {
                // Pre-calculate the entry to reuse
                let entry = DailyBalance {
                    date: current_date, // placeholder, updated in loop
                    balance: current_balance,
                    nisab_threshold: current_nisab,
                    is_above_nisab: current_balance >= current_nisab,
                };
                
                // Reserve space to avoid reallocs
                timeline.reserve(days_to_fill as usize);
                
                // Efficiently extend
                // Efficiently extend using iterator to allow bulk allocation
                timeline.extend((1..=days_to_fill).map(|i| {
                    let mut e = entry.clone();
                    e.date = current_date + Duration::days(i);
                    e
                }));
                
                // Advance current_date
                current_date += Duration::days(days_to_fill);
            }
        }
        
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

    #[test]
    fn test_performance_100_years() {
        // 100 Years of silence.
        // Should be instant due to jump logic.
        // If O(Days), it would iterate ~36,500 times. Not slow, but jump logic makes it O(1).
        
        let start_date = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2100, 1, 1).unwrap();
        
        let events = vec![
            LedgerEvent::new(start_date, dec!(10000), crate::types::WealthType::Business, TransactionType::Deposit, None),
        ]; // No other events
        
        // Mock price history with just one entry
        let mut prices = InMemoryPriceHistory::new();
        prices.add_price(start_date, dec!(1000));
        
        let start = std::time::Instant::now();
        let timeline = simulate_timeline(events, &prices, start_date, end_date).expect("Simulation failed");
        let duration = start.elapsed();
        
        println!("100 Years Simulation took: {:?}", duration);
        
        let days = (end_date - start_date).num_days() + 1;
        assert_eq!(timeline.len() as i64, days);
        assert!(duration.as_millis() < 500, "Simulation took too long: {:?} (Expected < 500ms)", duration);
    }
}
