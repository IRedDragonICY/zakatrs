/// Shared FFI-compatible types for Flutter Rust Bridge.
///
/// These types act as the bridge between Rust's strict types and Dart's type system.

use flutter_rust_bridge::frb;
use rust_decimal::prelude::*;
use anyhow::Result;

// ============================================================================
// FrbDecimal - Decimal wrapper for FFI
// ============================================================================

/// A wrapper around rust_decimal::Decimal for FFI compatibility.
/// 
/// This ensures financial precision is maintained across the Dart/Rust boundary.
/// Always use this type for monetary values instead of f64.
#[derive(Debug, Clone)]
pub struct FrbDecimal {
    pub(crate) value: Decimal,
}

impl FrbDecimal {
    /// Create from string representation (most precise).
    #[frb(sync)]
    pub fn from_string(s: String) -> Result<Self> {
        let d = zakat::inputs::IntoZakatDecimal::into_zakat_decimal(s.as_str())
            .map_err(|e| anyhow::anyhow!("Invalid decimal format '{}': {}", s, e))?;
        Ok(Self { value: d })
    }

    /// Create from integer value.
    #[frb(sync)]
    pub fn from_int(val: i64) -> Self {
        Self { value: Decimal::from(val) }
    }
    
    /// Create zero value.
    #[frb(sync)]
    pub fn zero() -> Self {
        Self { value: Decimal::ZERO }
    }

    /// Convert to string representation.
    #[frb(sync)]
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
    
    /// Convert to f64 (may lose precision - use only for display).
    #[frb(sync)]
    pub fn to_f64(&self) -> f64 {
        self.value.to_f64().unwrap_or(0.0)
    }
    
    /// Check if value is zero.
    #[frb(sync)]
    pub fn is_zero(&self) -> bool {
        self.value.is_zero()
    }
    
    /// Check if value is positive.
    #[frb(sync)]
    pub fn is_positive(&self) -> bool {
        self.value.is_sign_positive() && !self.value.is_zero()
    }
}

impl From<Decimal> for FrbDecimal {
    fn from(d: Decimal) -> Self {
        Self { value: d }
    }
}

impl From<FrbDecimal> for Decimal {
    fn from(d: FrbDecimal) -> Self {
        d.value
    }
}

// ============================================================================
// DartZakatConfig - Configuration wrapper
// ============================================================================

/// Zakat calculation configuration for Dart.
#[derive(Clone)]
pub struct DartZakatConfig {
    pub(crate) inner: zakat::config::ZakatConfig,
}

impl DartZakatConfig {
    /// Create a new configuration with required parameters.
    #[frb(sync)]
    pub fn new(
        gold_price: FrbDecimal, 
        silver_price: FrbDecimal, 
        madhab: String
    ) -> Result<Self> {
        use zakat::prelude::*;
        
        let madhab_enum = match madhab.to_lowercase().as_str() {
            "hanafi" => Madhab::Hanafi,
            "shafi" | "shafii" => Madhab::Shafi,
            "maliki" => Madhab::Maliki,
            "hanbali" => Madhab::Hanbali,
            _ => return Err(anyhow::anyhow!("Invalid madhab: {}. Use: hanafi, shafi, maliki, hanbali", madhab)),
        };

        let config = ZakatConfig::new()
            .with_madhab(madhab_enum)
            .with_gold_price(gold_price.value)
            .with_silver_price(silver_price.value);

        Ok(Self { inner: config })
    }
    
    /// Get the gold nisab threshold in currency.
    #[frb(sync)]
    pub fn gold_nisab(&self) -> FrbDecimal {
        let threshold = self.inner.gold_price_per_gram * self.inner.get_nisab_gold_grams();
        FrbDecimal { value: threshold }
    }
    
    /// Get the silver nisab threshold in currency.
    #[frb(sync)]
    pub fn silver_nisab(&self) -> FrbDecimal {
        let threshold = self.inner.silver_price_per_gram * self.inner.get_nisab_silver_grams();
        FrbDecimal { value: threshold }
    }
    
    /// Get the monetary nisab threshold (lower of gold/silver).
    #[frb(sync)]
    pub fn monetary_nisab(&self) -> FrbDecimal {
        FrbDecimal { value: self.inner.get_monetary_nisab_threshold() }
    }
}

// ============================================================================
// DartZakatResult - Calculation result
// ============================================================================

/// Zakat calculation result for Dart.
pub struct DartZakatResult {
    /// Amount of Zakat due.
    pub zakat_due: FrbDecimal,
    /// Whether Zakat is payable (net assets >= nisab).
    pub is_payable: bool,
    /// The nisab threshold used for this calculation.
    pub nisab_threshold: FrbDecimal,
    /// Net assets after deducting liabilities.
    pub net_assets: FrbDecimal,
    /// Total assets before deductions.
    pub total_assets: FrbDecimal,
    /// Type of wealth (e.g., "Business", "Gold").
    pub wealth_type: String,
    /// Optional label for the asset.
    pub label: Option<String>,
    /// Status reason if not payable.
    pub status_reason: Option<String>,
    /// Non-fatal warnings about the calculation.
    pub warnings: Vec<String>,
    /// Step-by-step calculation trace.
    pub calculation_trace: Vec<DartCalculationStep>,
}

impl DartZakatResult {
    /// Create from core ZakatDetails.
    pub(crate) fn from_core(details: zakat::types::ZakatDetails) -> Self {
        Self {
            zakat_due: FrbDecimal { value: details.zakat_due },
            is_payable: details.is_payable,
            nisab_threshold: FrbDecimal { value: details.nisab_threshold },
            net_assets: FrbDecimal { value: details.net_assets },
            total_assets: FrbDecimal { value: details.total_assets },
            wealth_type: format!("{:?}", details.wealth_type),
            label: details.label.clone(),
            status_reason: details.status_reason.clone(),
            warnings: details.warnings.clone(),
            calculation_trace: details.calculation_trace.iter().map(DartCalculationStep::from_core).collect(),
        }
    }
}

// ============================================================================
// DartCalculationStep - For calculation trace
// ============================================================================

/// A single step in the calculation trace.
#[derive(Clone)]
pub struct DartCalculationStep {
    /// The step key (for i18n).
    pub key: String,
    /// The description in English.
    pub description: String,
    /// The amount at this step (if applicable).
    pub amount: Option<FrbDecimal>,
    /// The operation type.
    pub operation: String,
}

impl DartCalculationStep {
    pub(crate) fn from_core(step: &zakat::types::CalculationStep) -> Self {
        Self {
            key: step.key.clone(),
            description: step.description.clone(),
            amount: step.amount.map(|d| FrbDecimal { value: d }),
            operation: format!("{:?}", step.operation),
        }
    }
}
