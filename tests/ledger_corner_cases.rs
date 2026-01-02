use chrono::{NaiveDate, Duration};
use rust_decimal_macros::dec;
use zakat::ledger::events::{LedgerEvent, TransactionType};
use zakat::ledger::pricing::InMemoryPriceHistory;
use zakat::ledger::timeline::simulate_timeline;
use zakat::ledger::analyzer::analyze_hawl;
use zakat::types::WealthType;

#[test]
fn test_price_spike_breach() {
    // Wealth stays constant at 10,000.
    // Nisab is usually 1,000.
    // Spike: On June 1st, Nisab spikes to 20,000 (Gold price shock).
    // This should break the Hawl.
    
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let spike_date = NaiveDate::from_ymd_opt(2023, 6, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    
    let events = vec![
        LedgerEvent::new(start_date, dec!(10000), WealthType::Business, TransactionType::Deposit, None),
    ];
    
    let mut prices = InMemoryPriceHistory::new();
    // Default low price
    for d in 0..400 {
        let date = start_date + Duration::days(d);
        prices.add_price(date, dec!(1000));
    }
    // Spike
    prices.add_price(spike_date, dec!(20000));
    
    let timeline = simulate_timeline(events, &prices, start_date, end_date).expect("Simulation success");
    let result = analyze_hawl(&timeline);
    
    // Hawl should have started AFTER the spike date (June 2nd presumably, or whenever Nisab fell back)
    // Actually, assumes Nisab falls back next day? My loop set 20k for just one day.
    // So June 1st: Is Above Nisab? 10k >= 20k -> False.
    // June 2nd: 10k >= 1k -> True. streak starts June 2.
    // End Date: Dec 31.
    // Days: June 2 to Dec 31 approx 210 days. < 354.
    // So Zakat NOT due.
    
    assert!(!result.is_due, "Zakat should not be due because Hawl was broken by price spike");
    assert_eq!(result.last_breach, Some(spike_date));
    assert!(result.current_streak_days < 354);
}

#[test]
fn test_leap_year_hawl() {
    // Test boundary of 354 days.
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    // 353 days later
    let date_353 = start_date + Duration::days(353);
    // 354 days later
    let _date_354 = start_date + Duration::days(354);
    
    let mut prices = InMemoryPriceHistory::new();
    for d in 0..400 {
        let date = start_date + Duration::days(d);
        prices.add_price(date, dec!(1000));
    }
    
    let events = vec![
        LedgerEvent::new(start_date, dec!(10000), WealthType::Business, TransactionType::Deposit, None),
    ];
    
    // Case A: 353 days (start to start+353 inclusive? No, diff is 353 days. 
    // timeline length is what matters. 
    // simulate_timeline includes end_date.
    // If start Jan 1, end Jan 1 -> 1 day.
    // We want 354 days length.
    
    let timeline_short = simulate_timeline(events.clone(), &prices, start_date, date_353).unwrap(); // Length 354 actually?
    // Jan 1 to Jan 2 is 2 days (Jan 1, Jan 2).
    // Jan 1 + 353 days = Dec 20 (approx). 
    // timeline.len() = 354.
    // analyze_hawl uses `days_held`.
    // If timeline is full of valid days, days_held = timeline.len().
    // So if len = 354 -> Is Due.
    
    // Wait, 354 days requirement typically means one FULL lunar year has passed.
    // Fiqh: Has 354 days passed? Yes.
    
    let result_short = analyze_hawl(&timeline_short); // 354 days of data?
    // days_held calculation: (today - start) + 1.
    // (date_353 - start) + 1 = 353 + 1 = 354.
    // So 353 days offset gives 354 days count.
    
    assert!(result_short.is_due, "Should be due if held for 354 days (inclusive)");
    
    // Let's try 352 offset -> 353 days count.
    let date_352 = start_date + Duration::days(352);
    let timeline_shorter = simulate_timeline(events.clone(), &prices, start_date, date_352).unwrap();
    let result_shorter = analyze_hawl(&timeline_shorter);
    assert!(!result_shorter.is_due, "Should NOT be due if held for 353 days");
}

#[test]
fn test_day_zero_deposit() {
    // Start with 0 balance.
    // Day 10: Deposit.
    // Hawl should start on Day 10.
    
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    let deposit_date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(); // Day 10
    let end_date = start_date + Duration::days(360);
    
    let events = vec![
        LedgerEvent::new(deposit_date, dec!(10000), WealthType::Business, TransactionType::Deposit, None),
    ];
    
    let mut prices = InMemoryPriceHistory::new();
    for d in 0..400 {
        let date = start_date + Duration::days(d);
        prices.add_price(date, dec!(1000));
    }
    
    let timeline = simulate_timeline(events.clone(), &prices, start_date, end_date).unwrap();
    let result = analyze_hawl(&timeline);
    
    // 352 days < 354 days, so NOT due yet.
    assert!(!result.is_due);
    assert_eq!(result.hawl_start_date, Some(deposit_date));
    
    // Check days held
    // From Jan 10 to (Jan 1 + 360)
    // Jan 1 + 9 = Jan 10.
    // End = Jan 1 + 360.
    // Days = (360 - 9) + 1? No.
    // (end - deposit) + 1 = (start+360 - (start+9)) + 1 = 360 - 9 + 1 = 352?
    // Wait. 360 - 9 = 351. +1 = 352.
    // 352 < 354.
    // So it should NOT be due yet?
    
    // Let's calculate exactly.
    // We want it to be due to verify Hawl Start Date logic mostly?
    // User said: "Day-0 Deposit: Start at 0, deposit 10k. Hawl must start on deposit date, not start_date."
    
    // If I extend end_date to be enough.
    // 9 + 354 = 363.
    // So if end_date is start + 365 (standard solar year), it should be due.
    // 360 is close.
    // Let's use 370 offset to be safe safe.
    
    let end_date_safe = start_date + Duration::days(370);
    let timeline_safe = simulate_timeline(events, &prices, start_date, end_date_safe).unwrap();
    let result_safe = analyze_hawl(&timeline_safe);
    
    assert!(result_safe.is_due);
    assert_eq!(result_safe.hawl_start_date, Some(deposit_date), "Hawl should start on deposit date");
}
