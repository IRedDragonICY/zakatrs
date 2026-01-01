use std::sync::Arc;
use std::str::FromStr;
use crate::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// UniFFI-compatible error type for Kotlin bindings.
#[derive(Debug, uniffi::Error)]
#[uniffi(flat_error)]
pub enum KotlinZakatError {
    /// Failed to parse a decimal value from string input.
    ParseError { field: String, message: String },
}

impl std::fmt::Display for KotlinZakatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError { field, message } => {
                write!(f, "Failed to parse '{}': {}", field, message)
            }
        }
    }
}

impl std::error::Error for KotlinZakatError {}

/// Helper function to parse a String to Decimal, returning a KotlinZakatError on failure.
fn parse_decimal(field: &str, value: &str) -> Result<Decimal, KotlinZakatError> {
    Decimal::from_str(value).map_err(|e| KotlinZakatError::ParseError {
        field: field.to_string(),
        message: format!("Invalid decimal '{}': {}", value, e),
    })
}

// --- Facade: Configuration ---
#[derive(uniffi::Record)]
pub struct KotlinZakatConfig {
    pub gold_price: String,
    pub silver_price: String,
}

#[derive(uniffi::Object)]
pub struct KotlinConfigWrapper {
    pub inner: ZakatConfig,
}

#[uniffi::export]
impl KotlinConfigWrapper {
    #[uniffi::constructor]
    pub fn new(gold_price: String, silver_price: String) -> Result<Arc<Self>, KotlinZakatError> {
        let gold = parse_decimal("gold_price", &gold_price)?;
        let silver = parse_decimal("silver_price", &silver_price)?;
        
        let cfg = ZakatConfig {
            gold_price_per_gram: gold,
            silver_price_per_gram: silver,
            ..Default::default()
        };
        Ok(Arc::new(Self { inner: cfg }))
    }
}

// --- Facade: Assets (Business) ---
#[derive(uniffi::Object)]
pub struct KotlinBusinessZakat {
    pub cash: Decimal,
    pub merchandise: Decimal,
    pub receivables: Decimal,
    pub debt: Decimal,
    pub expenses: Decimal,
}

#[uniffi::export]
impl KotlinBusinessZakat {
    #[uniffi::constructor]
    pub fn new(
        cash: String,
        merchandise: String,
        receivables: String,
        debt: String,
        expenses: String,
    ) -> Result<Arc<Self>, KotlinZakatError> {
        Ok(Arc::new(Self {
            cash: parse_decimal("cash", &cash)?,
            merchandise: parse_decimal("merchandise", &merchandise)?,
            receivables: parse_decimal("receivables", &receivables)?,
            debt: parse_decimal("debt", &debt)?,
            expenses: parse_decimal("expenses", &expenses)?,
        }))
    }

    pub fn calculate(&self, config: Arc<KotlinConfigWrapper>) -> f64 {
        let business = BusinessZakat::new()
            .cash(self.cash)
            .inventory(self.merchandise)
            .receivables(self.receivables)
            .debt(self.debt)
            .liabilities(self.expenses);
            
        match business.calculate_zakat(&config.inner) {
            Ok(result) => result.zakat_due.to_f64().unwrap_or(0.0),
            Err(_) => 0.0
        }
    }
}

// --- Facade: Assets (Gold) ---
#[derive(uniffi::Object)]
pub struct KotlinPreciousMetals {
    pub gold_grams: Decimal,
    pub silver_grams: Decimal,
}

#[uniffi::export]
impl KotlinPreciousMetals {
    #[uniffi::constructor]
    pub fn new(gold_grams: String, silver_grams: String) -> Result<Arc<Self>, KotlinZakatError> {
        Ok(Arc::new(Self {
            gold_grams: parse_decimal("gold_grams", &gold_grams)?,
            silver_grams: parse_decimal("silver_grams", &silver_grams)?,
        }))
    }

    pub fn calculate(&self, config: Arc<KotlinConfigWrapper>) -> f64 {
        let mut total_zakat = Decimal::ZERO;

        if self.gold_grams > Decimal::ZERO {
            let metals = PreciousMetals::new()
                .weight(self.gold_grams)
                .metal_type(WealthType::Gold);
            if let Ok(res) = metals.calculate_zakat(&config.inner) {
                total_zakat += res.zakat_due;
            }
        }

        if self.silver_grams > Decimal::ZERO {
            let metals = PreciousMetals::new()
                .weight(self.silver_grams)
                .metal_type(WealthType::Silver);
            if let Ok(res) = metals.calculate_zakat(&config.inner) {
                total_zakat += res.zakat_due;
            }
        }

        total_zakat.to_f64().unwrap_or(0.0)
    }
}
