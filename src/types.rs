use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the type of Zakat payment due.
///
/// This enum distinguishes between:
/// - **Monetary**: The default payment type, representing a currency value.
/// - **Livestock**: In-kind payment of specific animals (e.g., "1 Bint Makhad").
///   Used when Zakat is due as heads of livestock rather than cash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentPayload {
    /// Currency-based Zakat payment (default for most wealth types).
    Monetary(Decimal),
    /// In-kind livestock payment specifying animal types and counts.
    Livestock {
        description: String,
        heads_due: Vec<(String, u32)>, 
    },
    /// In-kind agriculture payment specifying harvest details.
    Agriculture {
        harvest_weight: Decimal,
        irrigation_method: String,
        crop_value: Decimal,
    },
}

/// Represents a single step in the Zakat calculation process.
///
/// This struct provides transparency into how the final Zakat amount was derived,
/// enabling users to understand and verify each step of the calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationStep {
    /// Human-readable description of what this step does.
    pub description: String,
    /// The value at this step (if applicable).
    pub amount: Option<Decimal>,
    /// The operation type: "Initial", "Add", "Subtract", "Compare", "Rate", "Result"
    pub operation: String,
}

impl CalculationStep {
    pub fn initial(description: impl Into<String>, amount: Decimal) -> Self {
        Self {
            description: description.into(),
            amount: Some(amount),
            operation: "Initial".to_string(),
        }
    }

    pub fn add(description: impl Into<String>, amount: Decimal) -> Self {
        Self {
            description: description.into(),
            amount: Some(amount),
            operation: "Add".to_string(),
        }
    }

    pub fn subtract(description: impl Into<String>, amount: Decimal) -> Self {
        Self {
            description: description.into(),
            amount: Some(amount),
            operation: "Subtract".to_string(),
        }
    }

    pub fn compare(description: impl Into<String>, amount: Decimal) -> Self {
        Self {
            description: description.into(),
            amount: Some(amount),
            operation: "compare".to_string(),
        }
    }

    pub fn rate(description: impl Into<String>, rate: Decimal) -> Self {
        CalculationStep {
            description: description.into(),
            amount: Some(rate),
            operation: "rate".to_string(),
        }
    }

    pub fn result(description: impl Into<String>, amount: Decimal) -> Self {
        CalculationStep {
            description: description.into(),
            amount: Some(amount),
            operation: "result".to_string(),
        }
    }

    pub fn info(description: impl Into<String>) -> Self {
        CalculationStep {
            description: description.into(),
            amount: None,
            operation: "info".to_string(),
        }
    }
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
    /// Step-by-step trace of how this calculation was derived.
    pub calculation_trace: Vec<CalculationStep>,
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

        // Build default calculation trace
        let mut trace = vec![
            CalculationStep::initial("Total Assets", total_assets),
            CalculationStep::subtract("Liabilities Due Now", liabilities_due_now),
            CalculationStep::result("Net Assets", net_assets),
            CalculationStep::compare("Nisab Threshold", nisab_threshold),
        ];
        if is_payable {
            trace.push(CalculationStep::rate("Applied Rate", rate));
            trace.push(CalculationStep::result("Zakat Due", zakat_due));
        } else {
            trace.push(CalculationStep::info("Net Assets below Nisab - No Zakat Due"));
        }

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
            calculation_trace: trace,
        }
    }

    /// Creates ZakatDetails with a custom calculation trace.
    /// Used by calculators that need more detailed step logging.
    pub fn with_trace(
        total_assets: Decimal,
        liabilities_due_now: Decimal,
        nisab_threshold: Decimal,
        rate: Decimal,
        wealth_type: WealthType,
        trace: Vec<CalculationStep>,
    ) -> Self {
        let net_assets = total_assets - liabilities_due_now;
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
            calculation_trace: trace,
        }
    }

    /// Helper to create a non-payable ZakatDetail because it is below the threshold.
    pub fn below_threshold(nisab_threshold: Decimal, wealth_type: WealthType, reason: &str) -> Self {
        let trace = vec![
            CalculationStep::info(reason.to_string()),
        ];
        
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
            calculation_trace: trace,
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

    /// Generates a human-readable explanation of the Zakat calculation.
    ///
    /// The output is formatted as a step-by-step list or table, showing operations
    /// and their results, helping users understand exactly how the `zakat_due` was determined.
    pub fn explain(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        let label = self.label.as_deref().unwrap_or("Asset");
        
        writeln!(&mut output, "Explanation for '{}' ({:?}):", label, self.wealth_type).unwrap();
        writeln!(&mut output, "{:-<50}", "").unwrap(); // Separator

        // Find the maximum description length for alignment
        let max_desc_len = self.calculation_trace.iter()
            .map(|step| step.description.len())
            .max()
            .unwrap_or(20)
            .max(20);

        for step in &self.calculation_trace {
            let op_symbol = match step.operation.as_str() {
                "Initial" => " ",
                "Add" => "+",
                "Subtract" => "-",
                "rate" => "x",
                "result" => "=",
                "compare" => "?",
                _ => " "
            };

            let amount_str = if let Some(amt) = step.amount {
                if step.operation == "rate" {
                     format!("{:.3}", amt) // Rates often have more precision e.g. 0.025
                } else {
                     format!("{:.2}", amt)
                }
            } else {
                String::new()
            };

            if step.operation == "info" {
                 writeln!(&mut output, "  INFO: {}", step.description).unwrap();
            } else if !amount_str.is_empty() {
                 writeln!(&mut output, "  {:<width$} : {} {:>10} ({})", 
                    step.description, 
                    op_symbol, 
                    amount_str, 
                    step.operation,
                    width = max_desc_len
                 ).unwrap();
            } else {
                 writeln!(&mut output, "  {:<width$} : [No Amount] ({})", 
                    step.description, 
                    step.operation,
                    width = max_desc_len
                 ).unwrap();
            }
        }
        
        writeln!(&mut output, "{:-<50}", "").unwrap();
        writeln!(&mut output, "Status: {}", if self.is_payable { "PAYABLE" } else { "EXEMPT" }).unwrap();
        if self.is_payable {
            writeln!(&mut output, "Amount Due: {}", self.format_amount()).unwrap();
        } else if let Some(reason) = &self.status_reason {
            writeln!(&mut output, "Reason: {}", reason).unwrap();
        }

        output
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
    Overflow {
        operation: String,
        source: Option<String>,
    },
    MissingConfig {
        field: String,
        source: Option<String>,
    },
}

impl ZakatError {
    pub fn with_source(self, source: String) -> Self {
        match self {
            ZakatError::CalculationError(msg, _) => ZakatError::CalculationError(msg, Some(source)),
            ZakatError::InvalidInput(msg, _) => ZakatError::InvalidInput(msg, Some(source)),
            ZakatError::ConfigurationError(msg, _) => ZakatError::ConfigurationError(msg, Some(source)),
            ZakatError::Overflow { operation, .. } => ZakatError::Overflow {
                operation,
                source: Some(source),
            },
            ZakatError::MissingConfig { field, .. } => ZakatError::MissingConfig {
                field,
                source: Some(source),
            },
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
            ZakatError::Overflow { operation, source } => {
                let s = source.as_deref().unwrap_or("Unknown");
                write!(f, "Arithmetic Overflow [Asset: {}]: Operation '{}' failed", s, operation)
            }
            ZakatError::MissingConfig { field, source } => {
                let s = source.as_deref().unwrap_or("Unknown");
                write!(f, "Missing Configuration [Asset: {}]: Field '{}' is required", s, field)
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
    pub fn Overflow(op: impl Into<String>) -> ZakatError {
        ZakatError::Overflow { operation: op.into(), source: None }
    }
    pub fn MissingConfig(field: impl Into<String>) -> ZakatError {
        ZakatError::MissingConfig { field: field.into(), source: None }
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
