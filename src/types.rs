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


/// Represents the semantic operation performed in a calculation step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    Initial,
    Add,
    Subtract,
    Multiply,
    Divide,
    Compare,
    Rate,
    Result,
    Info,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Operation::Initial => " ",
            Operation::Add => "+",
            Operation::Subtract => "-",
            Operation::Multiply => "*",
            Operation::Divide => "/",
            Operation::Compare => "?",
            Operation::Rate => "x",
            Operation::Result => "=",
            Operation::Info => "i",
        };
        write!(f, "{}", symbol)
    }
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
    /// The semantic operation type.
    pub operation: Operation,
}

impl CalculationStep {
    pub fn initial(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Initial,
        }
    }

    pub fn add(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Add,
        }
    }

    pub fn subtract(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Subtract,
        }
    }

    pub fn multiply(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
         Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Multiply,
        }
    }

    pub fn compare(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Compare,
        }
    }

    pub fn rate(description: impl Into<String>, rate: impl crate::inputs::IntoZakatDecimal) -> Self {
        CalculationStep {
            description: description.into(),
            amount: rate.into_zakat_decimal().ok(),
            operation: Operation::Rate,
        }
    }

    pub fn result(description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        CalculationStep {
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Result,
        }
    }

    pub fn info(description: impl Into<String>) -> Self {
        CalculationStep {
            description: description.into(),
            amount: None,
            operation: Operation::Info,
        }
    }
}

/// A collection of calculation steps that can be displayed or serialized.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalculationTrace(pub Vec<CalculationStep>);

impl std::ops::Deref for CalculationTrace {
    type Target = Vec<CalculationStep>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CalculationTrace {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Allow creating from Vec
impl From<Vec<CalculationStep>> for CalculationTrace {
    fn from(v: Vec<CalculationStep>) -> Self {
        CalculationTrace(v)
    }
}

// Enable iteration
impl IntoIterator for CalculationTrace {
    type Item = CalculationStep;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::fmt::Display for CalculationTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Find the maximum description length for alignment
        let max_desc_len = self.0.iter()
            .map(|step| step.description.len())
            .max()
            .unwrap_or(20)
            .max(20);

        for step in &self.0 {
            let op_symbol = step.operation.to_string();

            let amount_str = if let Some(amt) = step.amount {
                if matches!(step.operation, Operation::Rate) {
                     format!("{:.3}", amt)
                } else {
                     format!("{:.2}", amt)
                }
            } else {
                String::new()
            };

            if matches!(step.operation, Operation::Info) {
                 writeln!(f, "  INFO: {}", step.description)?;
            } else if !amount_str.is_empty() {
                 writeln!(f, "  {:<width$} : {} {:>10} ({:?})", 
                    step.description, 
                    op_symbol, 
                    amount_str, 
                    step.operation,
                    width = max_desc_len
                 )?;
            } else {
                 writeln!(f, "  {:<width$} : [No Amount] ({:?})", 
                    step.description, 
                    step.operation,
                    width = max_desc_len
                 )?;
            }
        }
        Ok(())
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
    pub calculation_trace: CalculationTrace,
    /// Non-fatal warnings about the calculation (e.g., negative values clamped).
    pub warnings: Vec<String>,
}

impl ZakatDetails {
    pub fn new(
        total_assets: Decimal,
        liabilities_due_now: Decimal,
        nisab_threshold: Decimal,
        rate: Decimal,
        wealth_type: WealthType,
    ) -> Self {
        let mut net_assets = total_assets - liabilities_due_now;
        let mut clamped_msg = None;
        let mut warnings = Vec::new();

        // Business rule: If net assets are negative, clamp to zero.
        if net_assets < Decimal::ZERO {
            net_assets = Decimal::ZERO;
            clamped_msg = Some("Net Assets are negative, clamped to zero for Zakat purposes");
            warnings.push("Net assets were negative and clamped to zero.".to_string());
        }

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
        ];

        if let Some(msg) = clamped_msg {
            trace.push(CalculationStep::info(msg));
        }

        trace.push(CalculationStep::result("Net Assets", net_assets));
        trace.push(CalculationStep::compare("Nisab Threshold", nisab_threshold));

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
            calculation_trace: CalculationTrace(trace),
            warnings,
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
        mut trace: Vec<CalculationStep>,
    ) -> Self {
        let mut net_assets = total_assets - liabilities_due_now;
        let mut warnings = Vec::new();
        
        if net_assets < Decimal::ZERO {
            net_assets = Decimal::ZERO;
            trace.push(CalculationStep::info("Net Assets are negative, clamped to zero for Zakat purposes"));
            warnings.push("Net assets were negative and clamped to zero.".to_string());
        }

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
            calculation_trace: CalculationTrace(trace),
            warnings,
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
            calculation_trace: CalculationTrace(trace),
            warnings: Vec::new(),
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
    /// If there are any warnings (e.g., negative values clamped), they are appended at the end.
    pub fn explain(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        let label = self.label.as_deref().unwrap_or("Asset");
        
        writeln!(&mut output, "Explanation for '{}' ({:?}):", label, self.wealth_type).unwrap();
        writeln!(&mut output, "{:-<50}", "").unwrap(); // Separator

        // Delegate trace printing to CalculationTrace
        write!(&mut output, "{}", self.calculation_trace).unwrap();
        
        writeln!(&mut output, "{:-<50}", "").unwrap();
        writeln!(&mut output, "Status: {}", if self.is_payable { "PAYABLE" } else { "EXEMPT" }).unwrap();
        if self.is_payable {
            writeln!(&mut output, "Amount Due: {}", self.format_amount()).unwrap();
        } else if let Some(reason) = &self.status_reason {
            writeln!(&mut output, "Reason: {}", reason).unwrap();
        }

        // Append warnings section if any
        if !self.warnings.is_empty() {
            writeln!(&mut output).unwrap();
            writeln!(&mut output, "WARNINGS:").unwrap();
            for warning in &self.warnings {
                writeln!(&mut output, " - {}", warning).unwrap();
            }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, thiserror::Error)]
pub enum ZakatError {
    #[error("Calculation error for '{source_label:?}': {reason}")]
    CalculationError {
        reason: String,
        source_label: Option<String>,
    },

    #[error("Invalid input for asset '{source_label:?}': Field '{field}' (value: '{value}') - {reason}")]
    InvalidInput {
        field: String,
        value: String,
        reason: String,
        source_label: Option<String>,
    },

    #[error("Configuration error for '{source_label:?}': {reason}")]
    ConfigurationError {
        reason: String,
        source_label: Option<String>,
    },
    
    #[error("Calculation overflow in '{operation}' for '{source_label:?}'")]
    Overflow {
        operation: String,
        source_label: Option<String>,
    },

    #[error("Missing configuration for '{source_label:?}': Field '{field}' is required")]
    MissingConfig {
        field: String,
        source_label: Option<String>,
    },
}

impl ZakatError {
    pub fn with_source(self, source: String) -> Self {
        match self {
            ZakatError::CalculationError { reason, .. } => ZakatError::CalculationError {
                reason,
                source_label: Some(source),
            },
            ZakatError::InvalidInput { field, value, reason, .. } => ZakatError::InvalidInput {
                field,
                value,
                reason,
                source_label: Some(source),
            },
            ZakatError::ConfigurationError { reason, .. } => ZakatError::ConfigurationError {
                reason,
                source_label: Some(source),
            },
            ZakatError::Overflow { operation, .. } => ZakatError::Overflow {
                operation,
                source_label: Some(source),
            },
            ZakatError::MissingConfig { field, .. } => ZakatError::MissingConfig {
                field,
                source_label: Some(source),
            },
        }
    }

    /// Generates a user-friendly error report.
    /// 
    /// Format includes:
    /// - The Asset Source (if available)
    /// - The Error Reason
    /// - A hinted remediation (if applicable)
    pub fn report(&self) -> String {
        let label = match self {
            ZakatError::CalculationError { source_label, .. } => source_label,
            ZakatError::InvalidInput { source_label, .. } => source_label,
            ZakatError::ConfigurationError { source_label, .. } => source_label,
            ZakatError::Overflow { source_label, .. } => source_label,
            ZakatError::MissingConfig { source_label, .. } => source_label,
        }.as_deref().unwrap_or("Unknown Source");

        let reason = match self {
            ZakatError::CalculationError { reason, .. } => reason.clone(),
            ZakatError::InvalidInput { field, value, reason, .. } => format!("Field '{}' has invalid value '{}' - {}", field, value, reason),
            ZakatError::ConfigurationError { reason, .. } => reason.clone(),
            ZakatError::Overflow { operation, .. } => format!("Overflow occurred during '{}'", operation),
            ZakatError::MissingConfig { field, .. } => format!("Missing required configuration field '{}'", field),
        };

        let hint = match self {
            ZakatError::ConfigurationError { reason, .. } => {
                if reason.contains("Gold price") || reason.contains("Silver price") {
                    "Suggestion: Set prices in ZakatConfig using .with_gold_price() / .with_silver_price()"
                } else {
                    "Suggestion: Check ZakatConfig setup."
                }
            },
            ZakatError::MissingConfig { field, .. } => {
                if field.contains("price") {
                     "Suggestion: Set missing price in ZakatConfig."
                } else {
                     "Suggestion: Ensure all required configuration fields are set."
                }
            },
            ZakatError::InvalidInput { .. } => "Suggestion: Ensure all input values are non-negative and correct.",
            _ => "Suggestion: Check input data accuracy."
        };

        format!(
            "Diagnostic Report:\n  Asset: {}\n  Error: {}\n  Hint: {}",
            label, reason, hint
        )
    }
}

// Removing ZakatErrorConstructors as we want to enforce structured creation


/// Helper enum to categorize wealth types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    Other(String),
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
