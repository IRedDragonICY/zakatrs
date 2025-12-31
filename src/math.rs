use rust_decimal::Decimal;
use crate::types::ZakatError;

/// A wrapper around `Decimal` that provides safe arithmetic operations with automatic
/// `ZakatError::Overflow` generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ZakatDecimal(pub Decimal);

impl ZakatDecimal {
    pub fn new(val: Decimal) -> Self {
        Self(val)
    }

    pub fn safe_add(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.0.checked_add(other_dec)
            .map(Self)
            .ok_or_else(|| ZakatError::Overflow {
                operation: "add".to_string(),
                source_label: None,
                asset_id: None,
            })
    }

    pub fn safe_sub(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.0.checked_sub(other_dec)
            .map(Self)
            .ok_or_else(|| ZakatError::Overflow {
                operation: "sub".to_string(),
                source_label: None,
                asset_id: None,
            })
    }

    pub fn safe_mul(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.0.checked_mul(other_dec)
            .map(Self)
            .ok_or_else(|| ZakatError::Overflow {
                operation: "mul".to_string(),
                source_label: None,
                asset_id: None,
            })
    }

    pub fn safe_div(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        if other_dec.is_zero() {
             return Err(ZakatError::CalculationError {
                reason: "Division by zero".to_string(),
                source_label: None,
                asset_id: None,
            });
        }
        self.0.checked_div(other_dec)
            .map(Self)
            .ok_or_else(|| ZakatError::Overflow {
                operation: "div".to_string(),
                source_label: None,
                asset_id: None,
            })
    }

    pub fn with_source(self, _label: Option<String>) -> Self {
         // This method is a bit of a placeholder since ZakatDecimal itself doesn't carry the source context
         // in arithmetic chain, but the Error does. Usually, errors are caught and enriched.
         // However, if we want to enrich the error *during* creation, we'd need to change how we construct errors
         // or catch them immediately. 
         // For the simple wrapper, we return the error from the op.
         self
    }
}

impl From<Decimal> for ZakatDecimal {
    fn from(d: Decimal) -> Self {
        Self(d)
    }
}

impl From<ZakatDecimal> for Decimal {
    fn from(val: ZakatDecimal) -> Self {
        val.0
    }
}

impl std::ops::Deref for ZakatDecimal {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
