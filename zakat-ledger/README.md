# zakat-ledger

Event sourcing, timeline simulation, and Hawl tracking for Zakat calculations.

## Overview

`zakat-ledger` implements the temporal aspects of Zakat:

- Hawl (lunar year) tracking with precise day counting
- Event-sourced wealth timeline reconstruction
- Nisab threshold monitoring over time
- Historical analysis for past Zakat obligations

## Hawl Tracking

The Hijri lunar year (Hawl) is 354 days. Zakat becomes obligatory when wealth remains above Nisab for one complete Hawl.

```rust
use zakat_ledger::hawl::{HawlTracker, HawlStatus};
use chrono::NaiveDate;

let mut tracker = HawlTracker::new();

// Record wealth above nisab
tracker.record_wealth(
    NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
    dec!(100000),
    dec!(50000), // nisab threshold
);

// Check status after one year
let status = tracker.status_on(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
match status {
    HawlStatus::Complete { days_held } => {
        println!("Hawl complete: {} days", days_held);
    }
    HawlStatus::InProgress { days_remaining } => {
        println!("Days remaining: {}", days_remaining);
    }
    HawlStatus::BelowNisab => {
        println!("Wealth below nisab, hawl reset");
    }
}
```

## Event Sourcing

Track wealth changes over time:

```rust
use zakat_ledger::{LedgerEvent, Timeline};

let mut timeline = Timeline::new();

timeline.add_event(LedgerEvent::Deposit {
    date: "2025-01-01".parse().unwrap(),
    amount: dec!(50000),
    asset_type: "cash".into(),
});

timeline.add_event(LedgerEvent::Withdrawal {
    date: "2025-06-15".parse().unwrap(),
    amount: dec!(10000),
    asset_type: "cash".into(),
});

// Reconstruct wealth at any point
let wealth = timeline.wealth_at("2025-06-30".parse().unwrap());
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `async` | Async ledger operations with Tokio |

## License

MIT
