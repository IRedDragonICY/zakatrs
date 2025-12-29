use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    /// Additional metadata for reporting (e.g. "2 Sheep due").
    pub extra_data: Option<std::collections::HashMap<String, String>>,
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
            extra_data: None,
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
            extra_data: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_extra_data(mut self, data: std::collections::HashMap<String, String>) -> Self {
        self.extra_data = Some(data);
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

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ZakatError {
    #[error("Calculation error: {0}")]
    CalculationError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Missing configuration: {0}")]
    ConfigurationError(String),
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
