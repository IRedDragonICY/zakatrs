//! SQLite-based persistence for Zakat Ledger events.
//!
//! This module provides a production-ready `SqliteStore` implementation
//! that persists ledger events to a SQLite database using `sqlx`.

use async_trait::async_trait;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

use zakat_core::types::{InvalidInputDetails, WealthType, ZakatError};
use zakat_ledger::events::{LedgerEvent, TransactionType};
use crate::persistence::LedgerStore;

/// A SQLite-backed implementation of `LedgerStore`.
///
/// Uses connection pooling via `sqlx::SqlitePool` for efficient concurrent access.
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    /// Creates a new SQLite store and ensures the schema is initialized.
    ///
    /// # Arguments
    /// * `db_url` - SQLite connection URL (e.g., `"sqlite::memory:"` or `"sqlite:ledger.db?mode=rwc"`)
    ///
    /// # Errors
    /// Returns `ZakatError::NetworkError` if connection or migration fails.
    pub async fn new(db_url: &str) -> Result<Self, ZakatError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await
            .map_err(|e| ZakatError::NetworkError(format!("SQLite connection error: {}", e)))?;

        let store = Self { pool };
        store.migrate().await?;
        Ok(store)
    }

    /// Creates a new `SqliteStore` from an existing pool.
    ///
    /// Note: This does NOT run migrations. Use `new()` for automatic schema setup.
    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Runs database migrations to ensure the schema exists.
    pub async fn migrate(&self) -> Result<(), ZakatError> {
        // 1. Create migrations table
        sqlx::query("CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)")
            .execute(&self.pool)
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Migration init error: {}", e)))?;

        // 2. Get current version
        let current_version: Option<i32> = sqlx::query_scalar("SELECT MAX(version) FROM _migrations")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Migration version check error: {}", e)))?;
            
        let version = current_version.unwrap_or(0);

        // 3. Apply migrations
        if version < 1 {
            sqlx::query(
                r#"
                CREATE TABLE IF NOT EXISTS ledger_events (
                    id TEXT PRIMARY KEY NOT NULL,
                    date TEXT NOT NULL,
                    amount TEXT NOT NULL,
                    asset_type TEXT NOT NULL,
                    transaction_type TEXT NOT NULL,
                    description TEXT
                )
                "#,
            )
            .execute(&self.pool)
            .await
            .map_err(|e| ZakatError::NetworkError(format!("Migration v1 error: {}", e)))?;
            
            sqlx::query("INSERT INTO _migrations (version) VALUES (1)")
                .execute(&self.pool)
                .await
                .map_err(|e| ZakatError::NetworkError(format!("Migration v1 version update error: {}", e)))?;
        }

        Ok(())
    }

    /// Returns a reference to the underlying connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait]
impl LedgerStore for SqliteStore {
    async fn save_event(&self, event: &LedgerEvent) -> Result<(), ZakatError> {
        let id = event.id.to_string();
        let date = event.date.format("%Y-%m-%d").to_string();
        let amount = event.amount.to_string();
        
        // Serialize enums to JSON for safe storage
        let asset_type = serde_json::to_string(&event.asset_type)
            .map_err(|e| make_serialize_error("asset_type", &e.to_string()))?;
        let transaction_type = serde_json::to_string(&event.transaction_type)
            .map_err(|e| make_serialize_error("transaction_type", &e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO ledger_events (id, date, amount, asset_type, transaction_type, description)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&date)
        .bind(&amount)
        .bind(&asset_type)
        .bind(&transaction_type)
        .bind(&event.description)
        .execute(&self.pool)
        .await
        .map_err(|e| ZakatError::NetworkError(format!("SQLite insert error: {}", e)))?;

        Ok(())
    }

    async fn load_events(&self) -> Result<Vec<LedgerEvent>, ZakatError> {
        let rows = sqlx::query(
            r#"
            SELECT id, date, amount, asset_type, transaction_type, description
            FROM ledger_events
            ORDER BY date ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ZakatError::NetworkError(format!("SQLite query error: {}", e)))?;

        let mut events = Vec::with_capacity(rows.len());

        for row in rows {
            let id_str: String = row.get("id");
            let date_str: String = row.get("date");
            let amount_str: String = row.get("amount");
            let asset_type_str: String = row.get("asset_type");
            let transaction_type_str: String = row.get("transaction_type");
            let description: Option<String> = row.get("description");

            let id = uuid::Uuid::parse_str(&id_str)
                .map_err(|e| make_parse_error("id", &id_str, &e.to_string()))?;

            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| make_parse_error("date", &date_str, &e.to_string()))?;

            let amount = rust_decimal::Decimal::from_str_exact(&amount_str)
                .map_err(|e| make_parse_error("amount", &amount_str, &e.to_string()))?;

            let asset_type: WealthType = serde_json::from_str(&asset_type_str)
                .map_err(|e| make_parse_error("asset_type", &asset_type_str, &e.to_string()))?;

            let transaction_type: TransactionType = serde_json::from_str(&transaction_type_str)
                .map_err(|e| make_parse_error("transaction_type", &transaction_type_str, &e.to_string()))?;

            events.push(LedgerEvent {
                id,
                date,
                amount,
                asset_type,
                transaction_type,
                description,
            });
        }

        Ok(events)
    }
}

fn make_serialize_error(field: &str, error: &str) -> ZakatError {
    ZakatError::InvalidInput(Box::new(InvalidInputDetails {
        code: zakat_core::types::ZakatErrorCode::InvalidInput,
        field: field.to_string(),
        value: "serialize".to_string(),
        reason_key: "error-serialize".to_string(),
        args: Some(std::collections::HashMap::from([
            ("error".to_string(), error.to_string()),
        ])),
        source_label: Some("SqliteStore".to_string()),
        ..Default::default()
    }))
}

fn make_parse_error(field: &str, value: &str, error: &str) -> ZakatError {
    ZakatError::InvalidInput(Box::new(InvalidInputDetails {
        code: zakat_core::types::ZakatErrorCode::InvalidInput,
        field: field.to_string(),
        value: value.to_string(),
        reason_key: "error-parse".to_string(),
        args: Some(std::collections::HashMap::from([
            ("error".to_string(), error.to_string()),
        ])),
        source_label: Some("SqliteStore".to_string()),
        ..Default::default()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_sqlite_store_roundtrip() {
        let store = SqliteStore::new("sqlite::memory:")
            .await
            .expect("Failed to connect to in-memory SQLite");

        let event = LedgerEvent::new(
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            dec!(5000.50),
            WealthType::Business,
            TransactionType::Deposit,
            Some("Initial deposit".to_string()),
        );

        store.save_event(&event).await.expect("Failed to save event");

        let loaded = store.load_events().await.expect("Failed to load events");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, event.id);
        assert_eq!(loaded[0].date, event.date);
        assert_eq!(loaded[0].amount, event.amount);
        assert_eq!(loaded[0].asset_type, event.asset_type);
        assert_eq!(loaded[0].transaction_type, event.transaction_type);
        assert_eq!(loaded[0].description, event.description);
    }

    #[tokio::test]
    async fn test_sqlite_store_ordered_by_date() {
        let store = SqliteStore::new("sqlite::memory:")
            .await
            .expect("Failed to connect");

        let event1 = LedgerEvent::new(
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            dec!(1000),
            WealthType::Gold,
            TransactionType::Deposit,
            None,
        );

        let event2 = LedgerEvent::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            dec!(2000),
            WealthType::Silver,
            TransactionType::Deposit,
            None,
        );

        store.save_event(&event1).await.unwrap();
        store.save_event(&event2).await.unwrap();

        let loaded = store.load_events().await.unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].date, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(loaded[1].date, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    }

    #[tokio::test]
    async fn test_sqlite_store_wealth_type_other() {
        let store = SqliteStore::new("sqlite::memory:")
            .await
            .expect("Failed to connect");

        let event = LedgerEvent::new(
            NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            dec!(3000),
            WealthType::Other("Cryptocurrency".to_string()),
            TransactionType::Income,
            Some("Bitcoin sale".to_string()),
        );

        store.save_event(&event).await.unwrap();
        let loaded = store.load_events().await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(
            loaded[0].asset_type,
            WealthType::Other("Cryptocurrency".to_string())
        );
    }
}
