use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEvent {
    pub id: Uuid,
    pub date: NaiveDate,
    pub amount: Decimal,
    pub asset_type: crate::types::WealthType,
    pub transaction_type: TransactionType,
    pub description: Option<String>,
}

impl LedgerEvent {
    pub fn new(
        date: NaiveDate,
        amount: Decimal,
        asset_type: crate::types::WealthType,
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

pub trait EventStream {
    fn get_events(&self) -> Vec<LedgerEvent>;
}
