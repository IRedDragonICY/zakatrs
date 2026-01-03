# zakat-sqlite

SQLite persistence layer for Zakat ledger and event storage.

## Overview

`zakat-sqlite` provides durable storage for:

- Ledger events and wealth snapshots
- Hawl tracking state
- Historical Zakat calculations
- Portfolio persistence

## Usage

```rust
use zakat_sqlite::SqliteStore;
use zakat_ledger::LedgerEvent;

// Initialize store
let store = SqliteStore::open("zakat.db").await?;

// Store an event
store.save_event(&LedgerEvent::Deposit {
    date: "2025-01-01".parse().unwrap(),
    amount: dec!(50000),
    asset_type: "cash".into(),
}).await?;

// Query events
let events = store.events_between(
    "2025-01-01".parse().unwrap(),
    "2025-12-31".parse().unwrap(),
).await?;
```

## Schema

The crate automatically manages schema migrations:

```sql
CREATE TABLE ledger_events (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    payload TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE wealth_snapshots (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    total_wealth TEXT NOT NULL,
    asset_breakdown TEXT NOT NULL
);
```

## Integration with Ledger

```rust
use zakat_sqlite::SqliteStore;
use zakat_ledger::Timeline;

// Load timeline from database
let store = SqliteStore::open("zakat.db").await?;
let events = store.all_events().await?;

let mut timeline = Timeline::new();
for event in events {
    timeline.add_event(event);
}
```

## Connection Pooling

For high-concurrency applications:

```rust
use zakat_sqlite::SqliteStore;

let store = SqliteStore::with_pool_size("zakat.db", 5).await?;
```

## Dependencies

- `sqlx` - Async SQLite driver
- `tokio` - Async runtime

## License

MIT
