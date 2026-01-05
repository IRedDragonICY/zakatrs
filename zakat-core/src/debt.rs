//! # Debt & Receivables Types (Dayn)
//!
//! Handles the classification of debts (receivables) for Zakat purposes.
//! Based on the strength of the expectation of repayment.

use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use rust_decimal::Decimal;

/// The quality of a receivable (Dayn), determining if it is Zakatable immediately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))] 
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum ReceivableQuality {
    /// **Marjuw al-Ada'** (Strong/Hopeful):
    /// The debtor adheres to the debt, acknowledges it, and is capable of paying.
    /// - **Ruling**: Zakatable immediately every year (Shafi'i/Hanbali/Modern consensus).
    /// - **Example**: Invoice to a reputable company, personal loan to a solvent friend.
    Strong,

    /// **Ghairu Marjuw** (Weak/Doubtful):
    /// The debtor is denying, delaying unreasonably, or is bankrupt/insolvent.
    /// - **Ruling**: Not Zakatable until actually received. Upon receipt, pay for one year (Maliki) or all years (Shafi'i option).
    /// - **Current Implementation**: Excluded from current Zakat base.
    /// - **Example**: Bad debts, loans to friends who have ghosted you.
    Weak,
}

/// A single receivable item with its quality classification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ReceivableItem {
    pub amount: Decimal,
    pub quality: ReceivableQuality,
    pub description: String,
}

impl ReceivableItem {
    pub fn new(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal, quality: ReceivableQuality) -> Self {
        Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().unwrap_or(Decimal::ZERO),
            quality,
        }
    }
}
