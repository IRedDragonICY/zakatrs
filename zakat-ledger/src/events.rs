//! Ledger Event Types
//!
//! Defines the transaction events used in ledger-based Zakat tracking.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zakat_core::types::WealthType;

/// Type of financial transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Income,
    Expense,
    Profit,
    Loss,
}

/// A single event in the ledger representing a financial transaction.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEvent {
    pub id: Uuid,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub asset_type: WealthType,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
}

impl LedgerEvent {
    pub fn new(
        date: NaiveDate,
        amount: Decimal,
        asset_type: WealthType,
        transaction_type: TransactionType,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            date,
            amount,
            asset_type,
            transaction_type,
            description,
        }
    }
}

/// Trait for types that can provide a stream of ledger events.
pub trait EventStream {
    fn get_events(&self) -> Vec<LedgerEvent>;
}
