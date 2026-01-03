//! Schema definitions for the compliance test suite.
//!
//! These structs define the JSON format of the golden data used by
//! polyglot test runners (Python, Dart, TypeScript).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The root compliance test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSuite {
    /// Suite metadata
    pub meta: SuiteMeta,
    /// List of test cases
    pub cases: Vec<TestCase>,
}

/// Metadata about the test suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteMeta {
    /// Version of the test suite schema
    pub schema_version: String,
    /// Timestamp when the suite was generated (ISO 8601)
    pub generated_at: String,
    /// Generator version (from Cargo.toml)
    pub generator_version: String,
    /// Total number of test cases
    pub total_cases: usize,
}

/// A single test case in the compliance suite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Unique test case identifier (e.g., "business_001")
    pub id: String,
    /// Human-readable description of what's being tested
    pub description: String,
    /// Category of the test (e.g., "happy_path", "edge_case", "error")
    pub category: TestCategory,
    /// The type of asset being tested
    pub asset_type: AssetType,
    /// Configuration for this test
    pub config: TestConfig,
    /// Input values for the asset
    pub input: TestInput,
    /// Expected result (calculated by Rust core)
    pub expected: ExpectedResult,
}

/// Category of test case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestCategory {
    /// Normal expected behavior
    HappyPath,
    /// Edge cases (zero values, boundary conditions)
    EdgeCase,
    /// Precision-related tests
    Precision,
    /// Configuration-related tests (madhab differences)
    Configuration,
    /// Expected error conditions
    Error,
    /// Input validation tests
    Validation,
}

/// Type of asset being tested.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Business,
    Gold,
    Silver,
    Income,
    Investment,
    Agriculture,
    Livestock,
}

/// Configuration for a test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Gold price per gram (string for precision)
    pub gold_price_per_gram: String,
    /// Silver price per gram (string for precision)
    pub silver_price_per_gram: String,
    /// Madhab to use (hanafi, shafi, maliki, hanbali)
    #[serde(default = "default_madhab")]
    pub madhab: String,
    /// Nisab standard override (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nisab_standard: Option<String>,
    /// Currency code
    #[serde(default = "default_currency")]
    pub currency_code: String,
}

fn default_madhab() -> String {
    "hanafi".to_string()
}

fn default_currency() -> String {
    "USD".to_string()
}

/// Input values for a test case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    /// Asset-specific inputs (all as strings for precision)
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
    /// Whether hawl is satisfied
    #[serde(default = "default_true")]
    pub hawl_satisfied: bool,
    /// Optional label for the asset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Liabilities due now
    #[serde(default)]
    pub liabilities_due_now: String,
}

fn default_true() -> bool {
    true
}

/// Expected result from the calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedResult {
    /// Whether zakat is payable
    pub is_payable: bool,
    /// Amount of zakat due (string for precision)
    pub zakat_due: String,
    /// Total assets (string for precision)
    pub total_assets: String,
    /// Net assets after liabilities (string for precision)
    pub net_assets: String,
    /// Nisab threshold used (string for precision)
    pub nisab_threshold: String,
    /// Expected error code (if this test expects an error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Expected error message pattern (if this test expects an error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message_contains: Option<String>,
    /// Warnings expected (if any)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl ExpectedResult {
    /// Creates a successful result.
    pub fn success(
        is_payable: bool,
        zakat_due: Decimal,
        total_assets: Decimal,
        net_assets: Decimal,
        nisab_threshold: Decimal,
    ) -> Self {
        Self {
            is_payable,
            zakat_due: zakat_due.to_string(),
            total_assets: total_assets.to_string(),
            net_assets: net_assets.to_string(),
            nisab_threshold: nisab_threshold.to_string(),
            error_code: None,
            error_message_contains: None,
            warnings: Vec::new(),
        }
    }

    /// Creates an error result.
    pub fn error(code: impl Into<String>) -> Self {
        Self {
            is_payable: false,
            zakat_due: "0".to_string(),
            total_assets: "0".to_string(),
            net_assets: "0".to_string(),
            nisab_threshold: "0".to_string(),
            error_code: Some(code.into()),
            error_message_contains: None,
            warnings: Vec::new(),
        }
    }

    /// Adds a warning to the result.
    #[allow(dead_code)] // Reserved for future test scenarios
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Adds an error message pattern.
    #[allow(dead_code)] // Reserved for future test scenarios
    pub fn with_error_contains(mut self, pattern: impl Into<String>) -> Self {
        self.error_message_contains = Some(pattern.into());
        self
    }
}

impl TestConfig {
    /// Creates a standard test config.
    pub fn standard(gold_price: &str, silver_price: &str) -> Self {
        Self {
            gold_price_per_gram: gold_price.to_string(),
            silver_price_per_gram: silver_price.to_string(),
            madhab: "hanafi".to_string(),
            nisab_standard: None,
            currency_code: "USD".to_string(),
        }
    }

    /// Sets the madhab.
    pub fn with_madhab(mut self, madhab: &str) -> Self {
        self.madhab = madhab.to_string();
        self
    }
}

impl TestInput {
    /// Creates a new test input.
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            hawl_satisfied: true,
            label: None,
            liabilities_due_now: "0".to_string(),
        }
    }

    /// Adds a field.
    pub fn field(mut self, name: &str, value: impl Into<serde_json::Value>) -> Self {
        self.fields.insert(name.to_string(), value.into());
        self
    }

    /// Sets hawl satisfied.
    pub fn hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    /// Sets liabilities.
    pub fn liabilities(mut self, amount: &str) -> Self {
        self.liabilities_due_now = amount.to_string();
        self
    }

    /// Sets a label.
    #[allow(dead_code)] // Reserved for future test scenarios
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
}

impl Default for TestInput {
    fn default() -> Self {
        Self::new()
    }
}
