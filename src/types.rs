use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentPayload {
    Monetary(Decimal),
    Livestock {
        description: String,
        heads_due: Vec<(String, u32)>, 
    },
}

/// Represents the detailed breakdown of the Zakat calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZakatDetails {
    /// Total assets subject to Zakat calculation.
    pub total_assets: Decimal,
    /// Liabilities that can be deducted from the total assets (Only debts due immediately).
    pub liabilities_due_now: Decimal,
    /// Net assets after deducting liabilities (total_assets - liabilities_due_now).
    pub net_assets: Decimal,
    /// The Nisab threshold applicable for this type of wealth.
    pub nisab_threshold: Decimal,
    /// Whether Zakat is due (net_assets >= nisab_threshold).
    pub is_payable: bool,
    /// The final Zakat amount due.
    pub zakat_due: Decimal,
    /// The type of wealth this calculation is for.
    pub wealth_type: WealthType,
    /// Reason for the status, if not payable (e.g. "Hawl not met").
    pub status_reason: Option<String>,
    /// Optional label for the asset (e.g. "Main Store", "Gold Necklace").
    pub label: Option<String>,
    /// Detailed payment payload (Monetary amount or specific assets like Livestock heads).
    pub payload: PaymentPayload,
}

impl ZakatDetails {
    pub fn new(
        total_assets: Decimal,
        liabilities_due_now: Decimal,
        nisab_threshold: Decimal,
        rate: Decimal,
        wealth_type: WealthType,
    ) -> Self {
        let net_assets = total_assets - liabilities_due_now;
        // Business rule: If net assets are negative, they are treated as zero for logic,
        // but it's good to preserve the actual value if needed.
        // For Nisab check: net_assets >= nisab_threshold
        let is_payable = net_assets >= nisab_threshold && net_assets > Decimal::ZERO;
        
        let zakat_due = if is_payable {
            net_assets * rate
        } else {
            Decimal::ZERO
        };

        ZakatDetails {
            total_assets,
            liabilities_due_now,
            net_assets,
            nisab_threshold,
            is_payable,
            zakat_due,
            wealth_type,
            status_reason: None,
            label: None,
            payload: PaymentPayload::Monetary(zakat_due),
        }
    }

    /// Helper to create a non-payable ZakatDetail because it is below the threshold.
    pub fn below_threshold(nisab_threshold: Decimal, wealth_type: WealthType, reason: &str) -> Self {
        ZakatDetails {
            total_assets: Decimal::ZERO,
            liabilities_due_now: Decimal::ZERO,
            net_assets: Decimal::ZERO,
            nisab_threshold,
            is_payable: false,
            zakat_due: Decimal::ZERO,
            wealth_type,
            status_reason: Some(reason.to_string()),
            label: None,
            payload: PaymentPayload::Monetary(Decimal::ZERO),
        }
    }

    pub fn with_payload(mut self, payload: PaymentPayload) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }



    /// Returns the Zakat due formatted as a string with 2 decimal places.
    pub fn format_amount(&self) -> String {
        use rust_decimal::RoundingStrategy;
        // Format with 2 decimal places
        let rounded = self.zakat_due.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
        format!("{:.2}", rounded)
    }

    /// Returns a concise status string.
    /// Format: "{Label}: {Payable/Exempt} - Due: {Amount}"
    /// Returns a concise status string.
    /// Format: "{Label}: {Payable/Exempt} - Due: {Amount}"
    pub fn summary(&self) -> String {
        let label_str = self.label.as_deref().unwrap_or("Asset");
        let status = if self.is_payable { "Payable" } else { "Exempt" };
        let reason = if let Some(r) = &self.status_reason {
             format!(" ({})", r)
        } else {
            String::new()
        };
        
        format!("{}: {}{} - Due: {}", label_str, status, reason, self.format_amount())
    }
}

impl std::fmt::Display for ZakatDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label_str = self.label.as_deref().unwrap_or("Asset");
        let type_str = format!("{:?}", self.wealth_type);
        
        writeln!(f, "Asset: {} (Type: {})", label_str, type_str)?;
        writeln!(f, "Net Assets: {} | Nisab: {}", self.net_assets, self.nisab_threshold)?;
        
        let status = if self.is_payable { "PAYABLE" } else { "EXEMPT" };
        let reason_str = self.status_reason.as_deref().unwrap_or("");
        
        if self.is_payable {
            write!(f, "Status: {} ({} due)", status, self.format_amount())
        } else {
            let reason_suffix = if !reason_str.is_empty() { format!(" - {}", reason_str) } else { String::new() };
            write!(f, "Status: {}{}", status, reason_suffix)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZakatError {
    CalculationError(String, Option<String>), // Msg, Source
    InvalidInput(String, Option<String>),
    ConfigurationError(String, Option<String>),
}

impl ZakatError {
    pub fn with_source(self, source: String) -> Self {
        match self {
            ZakatError::CalculationError(msg, _) => ZakatError::CalculationError(msg, Some(source)),
            ZakatError::InvalidInput(msg, _) => ZakatError::InvalidInput(msg, Some(source)),
            ZakatError::ConfigurationError(msg, _) => ZakatError::ConfigurationError(msg, Some(source)),
        }
    }
}

impl std::fmt::Display for ZakatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZakatError::CalculationError(msg, source) => {
                let s = source.as_deref().unwrap_or("Unknown");
                write!(f, "Calculation Error [Asset: {}]: {}", s, msg)
            }
            ZakatError::InvalidInput(msg, source) => {
                let s = source.as_deref().unwrap_or("Unknown");
                write!(f, "Invalid Input [Asset: {}]: {}", s, msg)
            }
            ZakatError::ConfigurationError(msg, source) => {
                let s = source.as_deref().unwrap_or("Unknown");
                write!(f, "Configuration Error [Asset: {}]: {}", s, msg)
            }
        }
    }
}

impl std::error::Error for ZakatError {}

/// Helper for backward compatibility or easy creation
#[allow(non_snake_case)]
pub mod ZakatErrorConstructors {
    use super::ZakatError;
    pub fn CalculationError(msg: impl Into<String>) -> ZakatError {
        ZakatError::CalculationError(msg.into(), None)
    }
    pub fn InvalidInput(msg: impl Into<String>) -> ZakatError {
        ZakatError::InvalidInput(msg.into(), None)
    }
    pub fn ConfigurationError(msg: impl Into<String>) -> ZakatError {
        ZakatError::ConfigurationError(msg.into(), None)
    }
}

/// Helper enum to categorize wealth types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WealthType {
    Fitrah,
    Gold,
    Silver,
    Business,
    Agriculture,
    Livestock,
    Income,
    Investment,
    Mining,
    Rikaz,
}

impl WealthType {
    /// Checks if the wealth type is considered "monetary" (Amwal Zakawiyyah)
    /// and should be aggregated for Nisab calculation under "Dam' al-Amwal".
    pub fn is_monetary(&self) -> bool {
        matches!(
            self,
            WealthType::Gold | WealthType::Silver | WealthType::Business | WealthType::Income | WealthType::Investment
        )
    }
}
