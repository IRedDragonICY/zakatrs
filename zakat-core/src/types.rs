use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Represents the age category of livestock for Zakat purposes.
/// 
/// These categories are based on the Hadith specification for camel and cattle ages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub enum LivestockAge {
    /// Bint Makhad - 1-year female camel (just weaned)
    BintMakhad,
    /// Bint Labun - 2-year female camel (mother nursing another)
    BintLabun,
    /// Hiqqah - 3-year female camel (ready for breeding)
    Hiqqah,
    /// Jaza'ah - 4-year female camel (mature)
    Jazaah,
    /// Tabi - 1-year calf (male or female cow)
    Tabi,
    /// Musinnah - 2-year cow (fully grown)
    Musinnah,
    /// Generic sheep/goat (age not differentiated in Fiqh for payment)
    Jadha,
}

/// Represents the kind of livestock for Zakat.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub enum LivestockKind {
    Sheep,
    Goat,
    Cow,
    Camel,
}

/// Represents a structured livestock payment item.
/// 
/// This struct enables frontends to translate livestock descriptions dynamically
/// instead of receiving pre-translated strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct LivestockDueItem {
    /// Number of animals due
    pub count: u32,
    /// Age category of the animal
    pub age: LivestockAge,
    /// Kind of animal
    pub kind: LivestockKind,
}

impl LivestockDueItem {
    /// Creates a new LivestockDueItem
    pub fn new(count: u32, age: LivestockAge, kind: LivestockKind) -> Self {
        Self { count, age, kind }
    }
    
    /// Returns a translation key for this livestock item.
    /// Frontends can use this key to fetch localized strings.
    pub fn translation_key(&self) -> String {
        let age_key = match self.age {
            LivestockAge::BintMakhad => "camel-age-bint-makhad",
            LivestockAge::BintLabun => "camel-age-bint-labun",
            LivestockAge::Hiqqah => "camel-age-hiqqah",
            LivestockAge::Jazaah => "camel-age-jazaah",
            LivestockAge::Tabi => "cow-age-tabi",
            LivestockAge::Musinnah => "cow-age-musinnah",
            LivestockAge::Jadha => "livestock-kind-sheep",
        };
        age_key.to_string()
    }
}

/// Represents the type of Zakat payment due.
///
/// This enum distinguishes between:
/// - **Monetary**: The default payment type, representing a currency value.
/// - **Livestock**: In-kind payment of specific animals with structured data for i18n.
///   Used when Zakat is due as heads of livestock rather than cash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentPayload {
    /// Currency-based Zakat payment (default for most wealth types).
    #[typeshare(serialized_as = "string")]
    Monetary(Decimal),
    /// In-kind livestock payment specifying animal types and counts.
    Livestock {
        /// Structured list of animals due (preferred for i18n)
        heads_due: Vec<LivestockDueItem>,
    },
    /// In-kind agriculture payment specifying harvest details.
    Agriculture {
        #[typeshare(serialized_as = "string")]
        harvest_weight: Decimal,
        irrigation_method: String,
        #[typeshare(serialized_as = "string")]
        crop_value: Decimal,
    },
}

impl PaymentPayload {
    /// Generates a human-readable description string from the livestock payment.
    /// Use this for display purposes; for i18n, iterate over `heads_due` directly.
    pub fn livestock_description(&self) -> Option<String> {
        match self {
            PaymentPayload::Livestock { heads_due } => {
                let parts: Vec<String> = heads_due.iter()
                    .map(|item| {
                        let name = match item.age {
                            LivestockAge::BintMakhad => "Bint Makhad",
                            LivestockAge::BintLabun => "Bint Labun",
                            LivestockAge::Hiqqah => "Hiqqah",
                            LivestockAge::Jazaah => "Jaza'ah",
                            LivestockAge::Tabi => "Tabi'",
                            LivestockAge::Musinnah => "Musinnah",
                            LivestockAge::Jadha => match item.kind {
                                LivestockKind::Sheep => "Sheep",
                                LivestockKind::Goat => "Goat",
                                _ => "Sheep/Goat",
                            },
                        };
                        format!("{} {}", item.count, name)
                    })
                    .collect();
                Some(parts.join(", "))
            }
            _ => None,
        }
    }
}


/// Represents the semantic operation performed in a calculation step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct CalculationStep {
    /// The Fluent ID (e.g., "step-net-assets").
    pub key: String,
    /// Fallback English text.
    pub description: String,
    /// The value at this step (if applicable).
    #[typeshare(serialized_as = "Option<string>")]
    pub amount: Option<Decimal>,
    /// The semantic operation type.
    pub operation: Operation,
    /// Variables for fluent.
    #[typeshare(skip)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct ZakatDetails {
    /// Total assets subject to Zakat calculation.
    #[typeshare(serialized_as = "string")]
    pub total_assets: Decimal,
    /// Liabilities that can be deducted from the total assets (Only debts due immediately).
    #[typeshare(serialized_as = "string")]
    pub liabilities_due_now: Decimal,
    /// Net assets after deducting liabilities (total_assets - liabilities_due_now).
    #[typeshare(serialized_as = "string")]
    pub net_assets: Decimal,
    /// The Nisab threshold applicable for this type of wealth.
    #[typeshare(serialized_as = "string")]
    pub nisab_threshold: Decimal,
    /// Whether Zakat is due (net_assets >= nisab_threshold).
    pub is_payable: bool,
    /// The final Zakat amount due.
    #[typeshare(serialized_as = "string")]
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
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct ZakatExplanation {
    /// Label of the asset (e.g., "Main Store", "Gold Necklace").
    pub label: String,
    /// Type of wealth (e.g., "Gold", "Business").
    pub wealth_type: String,
    /// Status of the calculation: "Payable" or "Exempt".
    pub status: String,
    /// The amount of Zakat due.
    #[typeshare(serialized_as = "string")]
    pub amount_due: Decimal,
    // [NEW] Unified View Model Fields
    /// Formatted total assets string.
    pub formatted_total: String,
    /// Formatted Zakat due string.
    pub formatted_due: String,
    /// Progress towards Nisab threshold (0.0 to 1.0).
    pub nisab_progress: f64,
    /// Currency code for the calculation.
    pub currency_code: String,
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

    /// Returns a concise status string (basic, non-localized).
    /// Format: "{Label}: {Payable/Exempt} - Due: {Amount}"
    /// 
    /// For localized output, use `zakat-i18n` crate.
    pub fn summary(&self) -> String {
        let label_str = self.label.as_deref().unwrap_or("Asset");
        let status = if self.is_payable { "PAYABLE" } else { "EXEMPT" };
        let reason = if let Some(r) = &self.status_reason {
             format!(" ({})", r)
        } else {
            String::new()
        };
        format!("{}: {}{} - Due: {:.2}", label_str, status, reason, self.zakat_due)
    }

    /// Converts this ZakatDetails into a structured `ZakatExplanation`.
    ///
    /// This is preferred for API consumers who want to render their own UI.
    pub fn to_explanation(&self, config: &crate::config::ZakatConfig) -> ZakatExplanation {
        let label = self.label.clone().unwrap_or_else(|| "Asset".to_string());
        let wealth_type = format!("{:?}", self.wealth_type);
        let status = if self.is_payable { "PAYABLE".to_string() } else { "EXEMPT".to_string() };
        
        let mut notes = Vec::new();
        if let Some(reason) = &self.status_reason {
            notes.push(reason.clone());
        }

        // Calculate Nisab Progress (0.0 to 1.0)
        let nisab_progress = if self.nisab_threshold > Decimal::ZERO {
            use rust_decimal::prelude::ToPrimitive;
            let ratio = (self.net_assets / self.nisab_threshold).to_f64().unwrap_or(0.0);
            ratio.clamp(0.0, 1.0)
        } else {
            1.0 // If nisab is 0, everything is zakatable -> 100% progress
        };

        ZakatExplanation {
            label,
            wealth_type,
            status,
            amount_due: self.zakat_due,
            formatted_total: config.format_currency(self.total_assets),
            formatted_due: config.format_currency(self.zakat_due),
            nisab_progress,
            currency_code: config.currency_code.clone(),
            steps: self.calculation_trace.0.clone(),
            warnings: self.warnings.clone(),
            notes,
        }
    }

    /// Generates a basic human-readable explanation of the Zakat calculation.
    ///
    /// For localized output, use `zakat-i18n` crate.
    pub fn explain(&self) -> String {
        format!("{}", self)
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
pub struct ErrorDetails {
    pub reason_key: String,
    pub args: Option<std::collections::HashMap<String, String>>,
    pub source_label: Option<String>,
    pub asset_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvalidInputDetails {
    pub field: String,
    pub value: String,
    pub reason_key: String,
    pub args: Option<std::collections::HashMap<String, String>>,
    pub source_label: Option<String>,
    pub asset_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, thiserror::Error)]
pub enum ZakatError {
    #[error("Calculation error: {0:?}")]
    CalculationError(Box<ErrorDetails>),

    #[error("Invalid input: {0:?}")]
    InvalidInput(Box<InvalidInputDetails>),

    #[error("Configuration error: {0:?}")]
    ConfigurationError(Box<ErrorDetails>),
    
    #[error("Calculation overflow in '{operation}'")]
    Overflow {
        operation: String,
        source_label: Option<String>,
        asset_id: Option<uuid::Uuid>,
    },

    #[error("Missing configuration: {field}")]
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
    /// Returns a standardized error code for FFI consumers.
    ///
    /// These codes are used by all FFI bindings (Python, Dart, WASM) to
    /// programmatically identify error types without parsing error messages.
    ///
    /// # Error Codes
    /// - `CALCULATION_ERROR`: Error during calculation logic
    /// - `INVALID_INPUT`: Invalid input value provided
    /// - `CONFIG_ERROR`: Configuration error (e.g., missing prices)
    /// - `MISSING_CONFIG`: Required configuration field is missing
    /// - `OVERFLOW`: Numeric overflow during calculation
    /// - `MULTIPLE_ERRORS`: Multiple validation errors occurred
    /// - `NETWORK_ERROR`: Network-related error (e.g., fetching prices)
    pub fn code(&self) -> &'static str {
        match self {
            ZakatError::CalculationError(_) => "CALCULATION_ERROR",
            ZakatError::InvalidInput(_) => "INVALID_INPUT",
            ZakatError::ConfigurationError(_) => "CONFIG_ERROR",
            ZakatError::MissingConfig { .. } => "MISSING_CONFIG",
            ZakatError::Overflow { .. } => "OVERFLOW",
            ZakatError::MultipleErrors(_) => "MULTIPLE_ERRORS",
            ZakatError::NetworkError(_) => "NETWORK_ERROR",
        }
    }

    pub fn with_source(self, source: String) -> Self {
        match self {
            ZakatError::CalculationError(mut details) => {
                details.source_label = Some(source);
                ZakatError::CalculationError(details)
            },
            ZakatError::InvalidInput(mut details) => {
                details.source_label = Some(source);
                ZakatError::InvalidInput(details)
            },
            ZakatError::ConfigurationError(mut details) => {
                details.source_label = Some(source);
                ZakatError::ConfigurationError(details)
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
            ZakatError::CalculationError(mut details) => {
                details.asset_id = Some(id);
                ZakatError::CalculationError(details)
            },
            ZakatError::InvalidInput(mut details) => {
                details.asset_id = Some(id);
                ZakatError::InvalidInput(details)
            },
            ZakatError::ConfigurationError(mut details) => {
                details.asset_id = Some(id);
                ZakatError::ConfigurationError(details)
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

    /// Reports the error as a user-friendly message (basic, non-localized).
    /// For localized output, use `zakat-i18n` crate.
    pub fn report(&self) -> String {
        match self {
            ZakatError::CalculationError(details) => {
                let msg = &details.reason_key;
                if let Some(lbl) = &details.source_label {
                    format!("{} (Asset: {})", msg, lbl)
                } else {
                    msg.clone()
                }
            },
            ZakatError::InvalidInput(details) => {
                format!("Invalid input for '{}': {} (Value: {})", details.field, details.reason_key, details.value)
            },
            ZakatError::ConfigurationError(details) => {
                details.reason_key.clone()
            },
            ZakatError::MissingConfig { field, .. } => {
                format!("Missing required configuration: {}", field)
            },
            ZakatError::Overflow { operation, .. } => {
                format!("Calculation overflow: {}", operation)
            },
            ZakatError::MultipleErrors(errs) => {
                let msgs: Vec<String> = errs.iter().map(|e| e.report()).collect();
                msgs.join("; ")
            },
            ZakatError::NetworkError(msg) => msg.clone(),
        }
    }

    /// Generates a user-friendly error report.
    pub fn report_default(&self) -> String {
        self.report()
    }

    fn get_hint(&self) -> &'static str {
         match self {
            ZakatError::ConfigurationError(details) => {
                if details.reason_key.contains("gold-price") || details.reason_key.contains("silver-price") {
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
            ZakatError::InvalidInput(_) => "Suggestion: Ensure all input values are non-negative and correct.",
            ZakatError::NetworkError(_) => "Suggestion: Check internet connection or API availability.",
            _ => "Suggestion: Check input data accuracy."
        }
    }

    /// Returns a structured JSON context for the error.
    /// Useful for WASM/Frontend consumers.
    pub fn context(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
             ZakatError::InvalidInput(details) => json!({
                 "code": "INVALID_INPUT",
                 "message": details.reason_key,
                 "field": details.field,
                 "value": details.value,
                 "source": details.source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::ConfigurationError(details) => json!({
                 "code": "CONFIG_ERROR",
                 "message": details.reason_key,
                 "source": details.source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::MissingConfig { field, source_label, .. } => json!({
                 "code": "MISSING_CONFIG",
                 "message": format!("Missing required field: {}", field),
                 "field": field,
                 "source": source_label,
                 "hint": self.get_hint()
             }),
             ZakatError::CalculationError(details) => json!({
                 "code": "CALCULATION_ERROR",
                 "message": details.reason_key,
                 "source": details.source_label,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, strum::Display, strum::EnumString, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
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

impl crate::inputs::ToFfiString for WealthType {
    fn to_ffi_string(&self) -> String {
        self.to_string()
    }
}

impl crate::inputs::FromFfiString for WealthType {
    type Err = strum::ParseError;
    fn from_ffi_string(s: &str) -> Result<Self, Self::Err> {
        use std::str::FromStr;
        Self::from_str(s)
    }
}

#[cfg(feature = "uniffi")]
use uniffi::Record;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "uniffi", derive(Record))]
pub struct FfiZakatDetails {
    pub total_assets: String,
    pub liabilities_due_now: String,
    pub net_assets: String,
    pub nisab_threshold: String,
    pub is_payable: bool,
    pub zakat_due: String,
    pub wealth_type: String, // debug formatted string
    pub status_reason: Option<String>,
    pub label: Option<String>,
    pub warnings: Vec<String>,
}

impl From<ZakatDetails> for FfiZakatDetails {
    fn from(src: ZakatDetails) -> Self {
        Self {
            total_assets: src.total_assets.to_string(),
            liabilities_due_now: src.liabilities_due_now.to_string(),
            net_assets: src.net_assets.to_string(),
            nisab_threshold: src.nisab_threshold.to_string(),
            is_payable: src.is_payable,
            zakat_due: src.zakat_due.to_string(),
            wealth_type: format!("{:?}", src.wealth_type),
            status_reason: src.status_reason,
            label: src.label,
            warnings: src.warnings,
        }
    }
}
// kept for backward compat if needed within file, or empty mod
#[cfg(feature = "uniffi")]
pub mod uniffi_types {
    pub type UniffiZakatDetails = super::FfiZakatDetails;
    pub type UniffiZakatError = super::FfiZakatError;
}

// =============================================================================
// FFI Error Type (Auto-generated projection for FFI boundaries)
// =============================================================================

/// Unified FFI-compatible error structure for WASM, UniFFI, and other FFI targets.
/// 
/// This struct eliminates the need for manual error type maintenance in each
/// FFI target module (wasm.rs, kotlin.rs, etc.). If you add a new variant to
/// `ZakatError`, the `From<ZakatError>` implementation below handles it automatically.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct FfiZakatError {
    /// Error code for programmatic handling (e.g., "CALCULATION_ERROR", "INVALID_INPUT")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// The field that caused the error (if applicable)
    pub field: Option<String>,
    /// Additional hint for fixing the error
    pub hint: Option<String>,
    /// Source label (which asset caused this error)
    pub source_label: Option<String>,
}

impl From<ZakatError> for FfiZakatError {
    fn from(err: ZakatError) -> Self {
        // Use default translator for message
        let message = err.report_default();
        
        match err {
            ZakatError::CalculationError(details) => FfiZakatError {
                code: "CALCULATION_ERROR".to_string(),
                message,
                field: None,
                hint: None,
                source_label: details.source_label,
            },
            ZakatError::InvalidInput(details) => FfiZakatError {
                code: "INVALID_INPUT".to_string(),
                message,
                field: Some(details.field),
                hint: Some(details.value),
                source_label: details.source_label,
            },
            ZakatError::ConfigurationError(details) => FfiZakatError {
                code: "CONFIG_ERROR".to_string(),
                message,
                field: None,
                hint: None,
                source_label: details.source_label,
            },
            ZakatError::MissingConfig { field, source_label, .. } => FfiZakatError {
                code: "MISSING_CONFIG".to_string(),
                message,
                field: Some(field),
                hint: None,
                source_label,
            },
            ZakatError::Overflow { operation, source_label, .. } => FfiZakatError {
                code: "OVERFLOW".to_string(),
                message,
                field: Some(operation),
                hint: None,
                source_label,
            },
            ZakatError::MultipleErrors(errs) => FfiZakatError {
                code: "MULTIPLE_ERRORS".to_string(),
                message: format!("{} errors occurred: {}", errs.len(), message),
                field: None,
                hint: None,
                source_label: None,
            },
            ZakatError::NetworkError(_) => FfiZakatError {
                code: "NETWORK_ERROR".to_string(),
                message,
                field: None,
                hint: None,
                source_label: None,
            },
        }
    }
}

// WASM-specific error conversion (requires "wasm" feature)
#[cfg(feature = "wasm")]
impl From<FfiZakatError> for wasm_bindgen::JsValue {
    fn from(err: FfiZakatError) -> Self {
        serde_wasm_bindgen::to_value(&err)
            .unwrap_or_else(|_| wasm_bindgen::JsValue::from_str(&err.message))
    }
}

// Helper to convert ZakatError directly to JsValue for WASM
#[cfg(feature = "wasm")]
impl From<ZakatError> for wasm_bindgen::JsValue {
    fn from(err: ZakatError) -> Self {
        let ffi_err: FfiZakatError = err.into();
        ffi_err.into()
    }
}
