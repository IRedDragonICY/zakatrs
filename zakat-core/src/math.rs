use rust_decimal::Decimal;
use crate::types::ZakatError; // Removed ErrorDetails as it was unused


/// A wrapper around `Decimal` that provides safe arithmetic operations with automatic
/// `ZakatError::Overflow` generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ZakatDecimal {
    pub value: Decimal,
    pub context: Option<String>,
}

impl ZakatDecimal {
    pub fn new(val: Decimal) -> Self {
        Self {
            value: val,
            context: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn checked_add(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.value.checked_add(other_dec)
            .map(|v| Self { value: v, context: self.context.clone() })
            .ok_or_else(|| ZakatError::Overflow {
                operation: "add".to_string(),
                source_label: self.context.clone(),
                asset_id: None,
            })
    }

    pub fn checked_sub(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.value.checked_sub(other_dec)
            .map(|v| Self { value: v, context: self.context.clone() })
            .ok_or_else(|| ZakatError::Overflow {
                operation: "sub".to_string(),
                source_label: self.context.clone(),
                asset_id: None,
            })
    }

    pub fn checked_mul(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        self.value.checked_mul(other_dec)
            .map(|v| Self { value: v, context: self.context.clone() })
            .ok_or_else(|| ZakatError::Overflow {
                operation: "mul".to_string(),
                source_label: self.context.clone(),
                asset_id: None,
            })
    }

    pub fn checked_div(self, other: impl Into<Decimal>) -> Result<Self, ZakatError> {
        let other_dec = other.into();
        if other_dec.is_zero() {
             return Err(ZakatError::Overflow {
                operation: "div (by zero)".to_string(),
                source_label: self.context.clone(),
                asset_id: None,
            });
        }
        self.value.checked_div(other_dec)
            .map(|v| Self { value: v, context: self.context.clone() })
            .ok_or_else(|| ZakatError::Overflow {
                operation: "div".to_string(),
                source_label: self.context.clone(),
                asset_id: None,
            })
    }

    pub fn with_source(self, label: Option<String>) -> Self {
        Self {
            value: self.value,
            context: label,
        }
    }
}

impl From<Decimal> for ZakatDecimal {
    fn from(d: Decimal) -> Self {
        Self::new(d)
    }
}

impl From<ZakatDecimal> for Decimal {
    fn from(val: ZakatDecimal) -> Self {
        val.value
    }
}

impl std::ops::Deref for ZakatDecimal {
    type Target = Decimal;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
