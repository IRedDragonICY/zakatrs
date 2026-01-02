use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::warn;

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
    /// The Fluent ID (e.g., "step-net-assets").
    pub key: String,
    /// Fallback English text.
    pub description: String,
    /// The value at this step (if applicable).
    pub amount: Option<Decimal>,
    /// The semantic operation type.
    pub operation: Operation,
    /// Variables for fluent.
    pub args: Option<std::collections::HashMap<String, String>>,
}

impl CalculationStep {
    pub fn initial(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Initial,
            args: None,
        }
    }

    pub fn add(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Add,
            args: None,
        }
    }

    pub fn subtract(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Subtract,
            args: None,
        }
    }

    pub fn multiply(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
         Self {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Multiply,
            args: None,
        }
    }

    pub fn compare(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Compare,
            args: None,
        }
    }

    pub fn rate(key: impl Into<String>, description: impl Into<String>, rate: impl crate::inputs::IntoZakatDecimal) -> Self {
        CalculationStep {
            key: key.into(),
            description: description.into(),
            amount: rate.into_zakat_decimal().ok(),
            operation: Operation::Rate,
            args: None,
        }
    }

    pub fn result(key: impl Into<String>, description: impl Into<String>, amount: impl crate::inputs::IntoZakatDecimal) -> Self {
        CalculationStep {
            key: key.into(),
            description: description.into(),
            amount: amount.into_zakat_decimal().ok(),
            operation: Operation::Result,
            args: None,
        }
    }

    pub fn info(key: impl Into<String>, description: impl Into<String>) -> Self {
        CalculationStep {
            key: key.into(),
            description: description.into(),
            amount: None,
            operation: Operation::Info,
            args: None,
        }
    }

    pub fn with_args(mut self, args: std::collections::HashMap<String, String>) -> Self {
        self.args = Some(args);
        self
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

/// Structured representation of a Zakat calculation for API consumers.
///
/// This struct allows frontend applications (e.g., React, Vue) to render their
/// own UI without parsing pre-formatted strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZakatExplanation {
    /// Label of the asset (e.g., "Main Store", "Gold Necklace").
    pub label: String,
    /// Type of wealth (e.g., "Gold", "Business").
    pub wealth_type: String,
    /// Status of the calculation: "Payable" or "Exempt".
    pub status: String,
    /// The amount of Zakat due.
    pub amount_due: Decimal,
    /// Step-by-step calculation steps.
    pub steps: Vec<CalculationStep>,
    /// Non-fatal warnings about the calculation.
    pub warnings: Vec<String>,
    /// Additional notes (e.g., exemption reason).
    pub notes: Vec<String>,
}

impl std::fmt::Display for ZakatExplanation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Explanation for '{}' ({}):", self.label, self.wealth_type)?;
        writeln!(f, "{:-<50}", "")?;

        // Print steps using CalculationTrace Display
        let trace = CalculationTrace(self.steps.clone());
        write!(f, "{}", trace)?;
        
        writeln!(f, "{:-<50}", "")?;
        writeln!(f, "Status: {}", self.status)?;
        
        if self.status == "PAYABLE" {
            writeln!(f, "Amount Due: {:.2}", self.amount_due)?;
        }
        
        for note in &self.notes {
            writeln!(f, "Reason: {}", note)?;
        }

        if !self.warnings.is_empty() {
            writeln!(f)?;
            writeln!(f, "WARNINGS:")?;
            for warning in &self.warnings {
                writeln!(f, " - {}", warning)?;
            }
        }

        Ok(())
    }
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
            warn!("Net assets were negative ({}), clamped to zero.", net_assets);
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
            CalculationStep::initial("step-total-assets", "Total Assets", total_assets),
            CalculationStep::subtract("step-liabilities", "Liabilities Due Now", liabilities_due_now),
        ];

        if let Some(msg) = clamped_msg {
            trace.push(CalculationStep::info("warn-negative-clamped", msg));
        }

        trace.push(CalculationStep::result("step-net-assets", "Net Assets", net_assets));
        trace.push(CalculationStep::compare("step-nisab-check", "Nisab Threshold", nisab_threshold));

        if is_payable {
            trace.push(CalculationStep::rate("step-rate-applied", "Applied Rate", rate));
            trace.push(CalculationStep::result("status-due", "Zakat Due", zakat_due));
        } else {
            trace.push(CalculationStep::info("status-exempt", "Net Assets below Nisab - No Zakat Due"));
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
            warn!("Net assets were negative ({}), clamped to zero.", net_assets);
            net_assets = Decimal::ZERO;
            trace.push(CalculationStep::info("warn-negative-clamped", "Net Assets are negative, clamped to zero for Zakat purposes"));
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
            CalculationStep::info("status-exempt", reason.to_string()),
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
    pub fn summary(&self, translator: &crate::i18n::Translator) -> String {
        self.summary_in(crate::i18n::ZakatLocale::EnUS, translator)
    }

    /// Returns a localized concise status string.
    pub fn summary_in(&self, locale: crate::i18n::ZakatLocale, translator: &crate::i18n::Translator) -> String {
        use crate::i18n::CurrencyFormatter;
            
        // Determine the label string.
        // If a custom label is provided, clone it.
        // Otherwise, fetch the localized generic "Asset" label from the translator.
        
        let label_string = if let Some(l) = &self.label {
            l.clone()
        } else {
            translator.translate(locale, "asset-generic", None)
        };
        let label_str = label_string.as_str();
        let status = if self.is_payable {
            translator.translate(locale, "status-payable", None)
        } else {
            translator.translate(locale, "status-exempt", None)
        };
        
        let due_label = translator.translate(locale, "status-due", None);
        let formatted_due = locale.format_currency(self.zakat_due);

        let reason = if let Some(r) = &self.status_reason {
             format!(" ({})", r)
        } else {
            String::new()
        };
        
        format!("{}: {}{} - {}: {}", label_str, status, reason, due_label, formatted_due)
    }

    /// Converts this ZakatDetails into a structured `ZakatExplanation`.
    ///
    /// This is preferred for API consumers who want to render their own UI.
    pub fn to_explanation(&self) -> ZakatExplanation {
        let label = self.label.clone().unwrap_or_else(|| "Asset".to_string());
        let wealth_type = format!("{:?}", self.wealth_type);
        let status = if self.is_payable { "PAYABLE".to_string() } else { "EXEMPT".to_string() };
        
        let mut notes = Vec::new();
        if let Some(reason) = &self.status_reason {
            notes.push(reason.clone());
        }

        ZakatExplanation {
            label,
            wealth_type,
            status,
            amount_due: self.zakat_due,
            steps: self.calculation_trace.0.clone(),
            warnings: self.warnings.clone(),
            notes,
        }
    }

    /// Generates a human-readable explanation of the Zakat calculation.
    ///
    /// The output is formatted as a step-by-step list or table, showing operations
    /// and their results, helping users understand exactly how the `zakat_due` was determined.
    /// If there are any warnings (e.g., negative values clamped), they are appended at the end.
    ///
    /// For structured data (e.g., for API consumers), use `to_explanation()` instead.
    pub fn explain(&self, translator: &crate::i18n::Translator) -> String {
        self.explain_in(crate::i18n::ZakatLocale::EnUS, translator)
    }

    /// Generates a localized human-readable explanation of the Zakat calculation.
    pub fn explain_in(&self, locale: crate::i18n::ZakatLocale, translator: &crate::i18n::Translator) -> String {
         use crate::i18n::CurrencyFormatter;
         use std::fmt::Write;

         let mut out = String::new();
         let label_string = if let Some(l) = &self.label {
             l.clone()
         } else {
             translator.translate(locale, "asset-generic", None)
         };
         let label_str = label_string.as_str();
         
         let type_str = format!("{:?}", self.wealth_type); // TODO: Localize wealth type
         
         writeln!(out, "{} ({})", label_str, type_str).ok();
         writeln!(out, "{:-<50}", "").ok();

         // Calculate max description length for alignment
         // We need to translate descriptions first to know length
         let mut localized_steps = Vec::new();
         for step in self.calculation_trace.iter() {
             let args = if let Some(args_map) = &step.args {
                 let mut f_args = fluent::FluentArgs::new();
                 for (k, v) in args_map {
                     f_args.set(k.as_str(), v.to_string());
                 }
                 Some(f_args)
             } else {
                 None
             };
             
             // Try to translate using key, fallback to direct description
             let desc = if !step.key.is_empty() {
                 let translated = translator.translate(locale, &step.key, args.as_ref());
                 if translated.starts_with("MISSING:") {
                     step.description.clone()
                 } else {
                     translated
                 }
             } else {
                 step.description.clone()
             };
             
             localized_steps.push((desc, step));
         }

         let max_desc_len = localized_steps.iter()
            .map(|(desc, _)| desc.chars().count())
            .max()
            .unwrap_or(20)
            .max(20);

         for (desc, step) in localized_steps {
            let op_symbol = step.operation.to_string();

            let amount_str = if let Some(amt) = step.amount {
                if matches!(step.operation, Operation::Rate) {
                     format!("{:.3}%", amt * rust_decimal_macros::dec!(100)) // Show rate as percentage? Or just raw
                } else {
                     locale.format_currency(amt)
                }
            } else {
                String::new()
            };

            if matches!(step.operation, Operation::Info) {
                 writeln!(out, "  INFO: {}", desc).ok();
            } else if !amount_str.is_empty() {
                 // Manual alignment
                 let padding = " ".repeat(max_desc_len.saturating_sub(desc.chars().count()));
                 writeln!(out, "  {}{} : {} {:>10} ({:?})", 
                    desc, 
                    padding,
                    op_symbol, 
                    amount_str, 
                    step.operation
                 ).ok();
            } else {
                 let padding = " ".repeat(max_desc_len.saturating_sub(desc.chars().count()));
                 writeln!(out, "  {}{} : [No Amount] ({:?})", 
                    desc, 
                    padding,
                    step.operation
                 ).ok();
            }
         }
         
         writeln!(out, "{:-<50}", "").ok();
         
         let status_key = if self.is_payable { "status-payable" } else { "status-exempt" };
         let status = translator.translate(locale, status_key, None);
         
         writeln!(out, "{}: {}", translator.translate(locale, "status-label", None), status).ok();
         
         if self.is_payable {
             let due_label = translator.translate(locale, "status-due", None);
             writeln!(out, "{}: {}", due_label, locale.format_currency(self.zakat_due)).ok();
         }
         
         if let Some(r) = &self.status_reason {
             writeln!(out, "Reason: {}", r).ok();
         }

         if !self.warnings.is_empty() {
             writeln!(out).ok();
             writeln!(out, "WARNINGS:").ok();
             for warning in &self.warnings {
                 writeln!(out, " - {}", warning).ok(); // TODO: Localize warnings
             }
         }

         out
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
        asset_id: Option<uuid::Uuid>,
    },

    #[error("Invalid input for asset '{source_label:?}': Field '{field}' (value: '{value}') - {reason}")]
    InvalidInput {
        field: String,
        value: String,
        reason: String,
        source_label: Option<String>,
        asset_id: Option<uuid::Uuid>,
    },

    #[error("Configuration error for '{source_label:?}': {reason}")]
    ConfigurationError {
        reason: String,
        source_label: Option<String>,
        asset_id: Option<uuid::Uuid>,
    },
    
    #[error("Calculation overflow in '{operation}' for '{source_label:?}'")]
    Overflow {
        operation: String,
        source_label: Option<String>,
        asset_id: Option<uuid::Uuid>,
    },

    #[error("Missing configuration for '{source_label:?}': Field '{field}' is required")]
    MissingConfig {
        field: String,
        source_label: Option<String>,
        asset_id: Option<uuid::Uuid>,
    },

    #[error("Multiple validation errors occurred")]
    MultipleErrors(Vec<ZakatError>),

    #[error("Network error: {0}")]
    NetworkError(String),
}

impl ZakatError {
    pub fn with_source(self, source: String) -> Self {
        match self {
            ZakatError::CalculationError { reason, asset_id, .. } => ZakatError::CalculationError {
                reason,
                source_label: Some(source),
                asset_id,
            },
            ZakatError::InvalidInput { field, value, reason, asset_id, .. } => ZakatError::InvalidInput {
                field,
                value,
                reason,
                source_label: Some(source),
                asset_id,
            },
            ZakatError::ConfigurationError { reason, asset_id, .. } => ZakatError::ConfigurationError {
                reason,
                source_label: Some(source),
                asset_id,
            },
            ZakatError::Overflow { operation, asset_id, .. } => ZakatError::Overflow {
                operation,
                source_label: Some(source),
                asset_id,
            },
            ZakatError::MissingConfig { field, asset_id, .. } => ZakatError::MissingConfig {
                field,
                source_label: Some(source.clone()),
                asset_id,
            },
            ZakatError::MultipleErrors(errors) => ZakatError::MultipleErrors(
                errors.into_iter().map(|e| e.with_source(source.clone())).collect()
            ),
            ZakatError::NetworkError(msg) => ZakatError::NetworkError(msg),
        }
    }

    /// Sets the asset ID for debugging purposes.
    pub fn with_asset_id(self, id: uuid::Uuid) -> Self {
        match self {
            ZakatError::CalculationError { reason, source_label, .. } => ZakatError::CalculationError {
                reason,
                source_label,
                asset_id: Some(id),
            },
            ZakatError::InvalidInput { field, value, reason, source_label, .. } => ZakatError::InvalidInput {
                field,
                value,
                reason,
                source_label,
                asset_id: Some(id),
            },
            ZakatError::ConfigurationError { reason, source_label, .. } => ZakatError::ConfigurationError {
                reason,
                source_label,
                asset_id: Some(id),
            },
            ZakatError::Overflow { operation, source_label, .. } => ZakatError::Overflow {
                operation,
                source_label,
                asset_id: Some(id),
            },
            ZakatError::MissingConfig { field, source_label, .. } => ZakatError::MissingConfig {
                field,
                source_label,
                asset_id: Some(id),
            },
            ZakatError::MultipleErrors(errors) => ZakatError::MultipleErrors(
                errors.into_iter().map(|e| e.with_asset_id(id)).collect()
            ),
            ZakatError::NetworkError(msg) => ZakatError::NetworkError(msg),
        }
    }

    /// Generates a user-friendly error report.
    /// 
    /// Format includes:
    /// - The Asset Source (if available)
    /// - The Asset ID (if available)
    /// - The Error Reason
    /// - A hinted remediation (if applicable)
    pub fn report(&self) -> String {
        // Handle MultipleErrors specially by combining reports
        if let ZakatError::MultipleErrors(errors) = self {
            let mut output = format!("Multiple Validation Errors ({} total):\n", errors.len());
            for (i, err) in errors.iter().enumerate() {
                output.push_str(&format!("\n--- Error {} ---\n{}", i + 1, err.report()));
            }
            return output;
        }

        let label = match self {
            ZakatError::CalculationError { source_label, .. } => source_label.as_deref(),
            ZakatError::InvalidInput { source_label, .. } => source_label.as_deref(),
            ZakatError::ConfigurationError { source_label, .. } => source_label.as_deref(),
            ZakatError::Overflow { source_label, .. } => source_label.as_deref(),
            ZakatError::MissingConfig { source_label, .. } => source_label.as_deref(),
            ZakatError::NetworkError(_) => Some("Network"),
            ZakatError::MultipleErrors(_) => unreachable!(), // Handled above
        }.unwrap_or("Unknown Source");

        let asset_id = match self {
            ZakatError::CalculationError { asset_id, .. } => asset_id.as_ref(),
            ZakatError::InvalidInput { asset_id, .. } => asset_id.as_ref(),
            ZakatError::ConfigurationError { asset_id, .. } => asset_id.as_ref(),
            ZakatError::Overflow { asset_id, .. } => asset_id.as_ref(),
            ZakatError::MissingConfig { asset_id, .. } => asset_id.as_ref(),
            ZakatError::NetworkError(_) => None,
            ZakatError::MultipleErrors(_) => unreachable!(), // Handled above
        };

        let reason = match self {
            ZakatError::CalculationError { reason, .. } => reason.clone(),
            ZakatError::InvalidInput { field, value, reason, .. } => format!("Field '{}' has invalid value '{}' - {}", field, value, reason),
            ZakatError::ConfigurationError { reason, .. } => reason.clone(),
            ZakatError::Overflow { operation, .. } => format!("Overflow occurred during '{}'", operation),
            ZakatError::MissingConfig { field, .. } => format!("Missing required configuration field '{}'", field),
            ZakatError::NetworkError(msg) => msg.clone(),
            ZakatError::MultipleErrors(_) => unreachable!(), // Handled above
        };

        let hint = self.get_hint();

        let id_str = asset_id.map(|id| format!("\n  Asset ID: {}", id)).unwrap_or_default();
        
        format!(
            "Diagnostic Report:\n  Asset: {}{}
  Error: {}\n  Hint: {}",
            label, id_str, reason, hint
        )
    }

    fn get_hint(&self) -> &'static str {
         match self {
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
            ZakatError::NetworkError(_) => "Suggestion: Check internet connection or API availability.",
            _ => "Suggestion: Check input data accuracy."
        }
    }

    /// Returns a structured JSON context for the error.
    /// Useful for WASM/Frontend consumers.
    pub fn context(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
             ZakatError::InvalidInput { field, value, reason, source_label, .. } => json!({
                 "code": "INVALID_INPUT",
                 "message": reason,
                 "field": field,
                 "value": value,
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::ConfigurationError { reason, source_label, .. } => json!({
                 "code": "CONFIG_ERROR",
                 "message": reason,
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::MissingConfig { field, source_label, .. } => json!({
                 "code": "MISSING_CONFIG",
                 "message": format!("Missing required field: {}", field),
                 "field": field,
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::CalculationError { reason, source_label, .. } => json!({
                 "code": "CALCULATION_ERROR",
                 "message": reason,
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::Overflow { operation, source_label, .. } => json!({
                 "code": "OVERFLOW",
                 "message": format!("Overflow in operation: {}", operation),
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::MultipleErrors(errors) => json!({
                 "code": "MULTIPLE_ERRORS",
                 "message": "Multiple validation errors occurred",
                 "errors": errors.iter().map(|e| e.context()).collect::<Vec<_>>()
             }),
             ZakatError::NetworkError(msg) => json!({
                 "code": "NETWORK_ERROR",
                 "message": msg,
                 "hint": self.get_hint()
             })
        }
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
