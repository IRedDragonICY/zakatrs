//! Zakat Configuration
//!
//! This module provides the core configuration struct for Zakat calculations.
//! I18n-specific features are provided by the `zakat-i18n` crate.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::sync::Arc;
use crate::types::{ZakatError, ErrorDetails};
use crate::inputs::IntoZakatDecimal;
use tracing::{instrument, debug};

use crate::madhab::{Madhab, NisabStandard, ZakatStrategy};

/// Controls validation strictness for Fiqh compliance.
///
/// This mode affects how the library handles missing data, edge cases,
/// and approximations during Zakat calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ZakatMode {
    /// Strict Fiqh compliance - all validations enforced.
    /// 
    /// Use this mode when you need precise, auditable calculations.
    /// Missing required data will result in errors.
    #[default]
    Strict,
    /// Permissive estimation - allows approximations and missing data.
    /// 
    /// Use this mode for quick estimates or when exact data is unavailable.
    /// Missing optional data uses safe defaults, unknown dates assume Hawl satisfied.
    Permissive,
}

/// Default strategy for serde deserialization.
fn default_strategy() -> Arc<dyn ZakatStrategy> {
    Arc::new(Madhab::default())
}


/// Networking configuration for external API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct NetworkConfig {
    /// Optional direct IP for Binance API to bypass DNS issues.
    /// If None, standard DNS resolution is used.
    pub binance_api_ip: Option<String>,
    
    /// Request timeout in seconds. Default: 30.
    pub timeout_seconds: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            binance_api_ip: None,
            timeout_seconds: 30,
        }
    }
}

impl NetworkConfig {
    /// Sets the Binance API IP address from a string.
    pub fn with_binance_ip(mut self, ip: impl Into<String>) -> Self {
        self.binance_api_ip = Some(ip.into());
        self
    }
}

/// Global configuration for Zakat prices.
#[derive(Clone, Serialize, Deserialize)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct ZakatConfig {
    /// The Zakat calculation strategy. Uses `Madhab::Hanafi` by default.
    /// Can be set to any custom strategy implementing `ZakatStrategy`.
    #[serde(skip, default = "default_strategy")]
    #[typeshare(skip)]
    pub strategy: Arc<dyn ZakatStrategy>,
    /// Current market price of Gold per gram.
    #[typeshare(serialized_as = "string")]
    pub gold_price_per_gram: Decimal,
    /// Current market price of Silver per gram.
    #[typeshare(serialized_as = "string")]
    pub silver_price_per_gram: Decimal,
    /// Price of Rice per kg (for Zakat Fitrah).
    #[typeshare(serialized_as = "Option<string>")]
    pub rice_price_per_kg: Option<Decimal>,
    /// Price of Rice per liter (for Zakat Fitrah).
    #[typeshare(serialized_as = "Option<string>")]
    pub rice_price_per_liter: Option<Decimal>,
    
    /// Nisab standard to use for cash, business assets, and investments.
    /// Set automatically via `with_madhab()` or manually via `with_nisab_standard()`.
    pub cash_nisab_standard: NisabStandard,
    
    // Custom Thresholds (Optional override, defaults provided)
    /// Override default Gold Nisab (default: 85g).
    #[typeshare(serialized_as = "Option<string>")]
    pub nisab_gold_grams: Option<Decimal>,
    /// Override default Silver Nisab (default: 595g).
    #[typeshare(serialized_as = "Option<string>")]
    pub nisab_silver_grams: Option<Decimal>,
    /// Override default Agriculture Nisab (default: 653kg).
    #[typeshare(serialized_as = "Option<string>")]
    pub nisab_agriculture_kg: Option<Decimal>,

    /// Locale code for output formatting (e.g., "en-US", "ar-SA").
    /// Use `zakat-i18n` crate for full i18n support.
    #[serde(default = "default_locale_code")]
    pub locale_code: String,

    /// Currency code (e.g., "USD", "SAR").
    #[serde(default = "default_currency_code")]
    pub currency_code: String,

    /// Network configuration for external API calls.
    #[serde(default)]
    pub networking: NetworkConfig,

    /// Validation mode controlling strictness of Fiqh compliance.
    /// Defaults to `Strict` for production use.
    #[serde(default)]
    pub mode: ZakatMode,
}

fn default_locale_code() -> String {
    "en-US".to_string()
}

fn default_currency_code() -> String {
    "USD".to_string()
}

// Manual Debug impl since Arc<dyn Trait> doesn't auto-derive Debug
impl std::fmt::Debug for ZakatConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ZakatConfig")
            .field("strategy", &self.strategy)
            .field("gold_price_per_gram", &self.gold_price_per_gram)
            .field("silver_price_per_gram", &self.silver_price_per_gram)
            .field("cash_nisab_standard", &self.cash_nisab_standard)
            .field("locale_code", &self.locale_code)
            .field("currency_code", &self.currency_code)
            .field("mode", &self.mode)
            .finish()
    }
}

impl Default for ZakatConfig {
    fn default() -> Self {
        ZakatConfig {
            strategy: Arc::new(Madhab::default()),
            gold_price_per_gram: Decimal::ZERO,
            silver_price_per_gram: Decimal::ZERO,
            rice_price_per_kg: None,
            rice_price_per_liter: None,
            cash_nisab_standard: NisabStandard::default(),
            nisab_gold_grams: None,
            nisab_silver_grams: None,
            nisab_agriculture_kg: None,
            locale_code: default_locale_code(),
            currency_code: default_currency_code(),
            networking: NetworkConfig::default(),
            mode: ZakatMode::default(),
        }
    }
}

// Ensure the caller can easily create a config
impl std::str::FromStr for ZakatConfig {
    type Err = ZakatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
            .map_err(|_e| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-parse-json".to_string(),
                suggestion: Some("Ensure the JSON is valid and matches the ZakatConfig schema.".to_string()),
                ..Default::default()
            })))
    }
}

impl ZakatConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration with default values suitable for testing.
    /// 
    /// This sets arbitrary valid prices so that `validate()` passes immediately,
    /// removing friction for quick prototypes.
    /// 
    /// Prices: Gold=$85.0, Silver=$1.0. Nisab Standard: Gold.
    pub fn test_default() -> Self {
        Self::new()
            .with_gold_price(dec!(85.0))
            .with_silver_price(dec!(1.0))
            .with_nisab_standard(NisabStandard::Gold)
    }

    /// Creates a pre-configured ZakatConfig for the Hanafi Madhab.
    /// Sets strategy to Hanafi and Nisab standard to LowerOfTwo (typically Silver).
    pub fn hanafi(gold_price: impl IntoZakatDecimal, silver_price: impl IntoZakatDecimal) -> Self {
        let gold = gold_price.into_zakat_decimal().unwrap_or(Decimal::ZERO);
        let silver = silver_price.into_zakat_decimal().unwrap_or(Decimal::ZERO);
        
        Self::new()
            .with_madhab(Madhab::Hanafi)
            .with_nisab_standard(NisabStandard::LowerOfTwo)
            .with_gold_price(gold)
            .with_silver_price(silver)
    }

    /// Creates a pre-configured ZakatConfig for the Shafi Madhab.
    /// Sets strategy to Shafi and Nisab standard to Gold.
    pub fn shafi(gold_price: impl IntoZakatDecimal) -> Self {
        let gold = gold_price.into_zakat_decimal().unwrap_or(Decimal::ZERO);
        
        Self::new()
            .with_madhab(Madhab::Shafi)
            .with_nisab_standard(NisabStandard::Gold)
            .with_gold_price(gold)
    }

    /// Initializes a `ZakatConfig` with regional Fiqh defaults and locale settings.
    /// 
    /// This method provides sensible presets based on dominant schools of jurisprudence (Madhab)
    /// and prevailing Nisab standards in specific regions.
    /// 
    /// # Regional Defaults
    /// - **South East Asia (ID, MY, SG, BN)**: Defaults to the **Shafi** Madhab and **Gold** Nisab standard, 
    ///   reflecting common practice in the Nusantara region.
    /// - **South Asia (PK, IN, BD, AF)**: Defaults to the **Hanafi** Madhab and **Silver** (LowerOfTwo) 
    ///   standard to ensure broader inclusion of those eligible to pay.
    /// - **Turkey (TR)**: Defaults to the **Hanafi** Madhab with a **Gold** Nisab, aligning with 
    ///   Diyanet's official annual guidance.
    /// - **GCC/Arab World**: Defaults to **Gold** Nisab, which is widely adopted for currency-based 
    ///   Zakat in modern stable economies.
    /// 
    /// # Example
    /// ```rust,ignore
    /// let config = ZakatConfig::for_region("MY")
    ///     .with_gold_price(300);
    /// ```
    pub fn for_region(iso_code: &str) -> Self {
        let code = iso_code.to_uppercase();
        match code.as_str() {
            // South East Asia -> Shafi (Gold)
            "ID" | "MY" | "SG" | "BN" => {
                Self::new()
                    .with_madhab(Madhab::Shafi)
                    .with_nisab_standard(NisabStandard::Gold)
                    .with_locale_code(match code.as_str() {
                        "ID" => "id-ID",
                        "MY" => "ms-MY",
                        _ => "en-SG"
                    })
                    .with_currency_code(match code.as_str() {
                        "ID" => "IDR",
                        "MY" => "MYR",
                        "SG" => "SGD",
                         "BN" => "BND",
                        _ => "USD"
                    })
            },
            // South Asia -> Hanafi (Silver/LowerOfTwo)
            "PK" | "IN" | "BD" | "AF" => {
                 Self::new()
                    .with_madhab(Madhab::Hanafi)
                    .with_nisab_standard(NisabStandard::LowerOfTwo) // Safer for poor
                    .with_locale_code(match code.as_str() {
                        "PK" => "ur-PK",
                        "BD" => "bn-BD",
                        _ => "en-IN"
                    })
                    .with_currency_code(match code.as_str() {
                        "PK" => "PKR",
                        "IN" => "INR",
                        "BD" => "BDT",
                        "AF" => "AFN",
                        _ => "USD"
                    })
            },
            // Turkey -> Hanafi (Gold - Diyanet standard)
            "TR" => {
                Self::new()
                    .with_madhab(Madhab::Hanafi)
                    .with_nisab_standard(NisabStandard::Gold)
                    .with_locale_code("tr-TR")
                    .with_currency_code("TRY")
            }
            // Arab World -> Defaulting to standard settings (often Gold or Silver)
            "SA" | "AE" | "QA" | "KW" => {
                 Self::new()
                    .with_madhab(Madhab::Hanafi) // Dominant but Hanbali also common
                    .with_nisab_standard(NisabStandard::Gold)
                    .with_locale_code("ar-SA")
                    .with_currency_code(match code.as_str() {
                        "AE" => "AED",
                        "QA" => "QAR",
                        "KW" => "KWD",
                         _ => "SAR"
                    })
            }
            // Western World -> Gold is common to avoid excessive burden on small savings
            "US" | "UK" | "CA" | "AU" | "NZ" | "FR" | "DE" => {
                 Self::new()
                    .with_madhab(Madhab::Hanafi)
                    .with_nisab_standard(NisabStandard::Gold)
                    .with_currency_code(match code.as_str() {
                        "UK" => "GBP",
                        "EU" | "DE" | "FR" => "EUR",
                        _ => "USD"
                    })
            }
            _ => Self::default()
        }
    }

    /// Finalizes the configuration and runs validation.
    /// 
    /// This is the recommended way to finish a ZakatConfig builder chain.
    pub fn build(self) -> Result<Self, ZakatError> {
        self.validate()?;
        Ok(self)
    }

    /// Validates the configuration for logical consistency and safety.
    #[instrument(skip(self))]
    pub fn validate(&self) -> Result<(), ZakatError> {
        if self.gold_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigMissing,
                reason_key: "error-config-gold-positive".to_string(),
                suggestion: Some("Run with --gold-price X or set ZAKAT_GOLD_PRICE env var.".to_string()),
                ..Default::default()
            })));
        }
        if self.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigMissing,
                reason_key: "error-config-silver-positive".to_string(),
                suggestion: Some("Run with --silver-price X or set ZAKAT_SILVER_PRICE env var.".to_string()),
                ..Default::default()
            })));
        }

        // Validation Logic based on Nisab Standard
        match self.cash_nisab_standard {
            NisabStandard::Gold => {
                 // Requires Gold price
            }
            NisabStandard::LowerOfTwo => {
                 // Requires both Gold and Silver prices to determine the lower threshold
            }
            _ => {}
        }
        
        if self.cash_nisab_standard == NisabStandard::Gold && self.gold_price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                 code: crate::types::ZakatErrorCode::ConfigMissing,
                 reason_key: "error-config-gold-positive".to_string(),
                suggestion: Some("Run with --gold-price X or set ZAKAT_GOLD_PRICE env var.".to_string()),
                 ..Default::default()
             })));
        }

        if self.cash_nisab_standard == NisabStandard::Silver && self.silver_price_per_gram <= Decimal::ZERO {
             return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                 code: crate::types::ZakatErrorCode::ConfigMissing,
                 reason_key: "error-config-silver-positive".to_string(),
                suggestion: Some("Run with --silver-price X or set ZAKAT_SILVER_PRICE env var.".to_string()),
                 ..Default::default()
             })));
        }

        if self.cash_nisab_standard == NisabStandard::LowerOfTwo {
            if self.gold_price_per_gram <= Decimal::ZERO {
                return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                    code: crate::types::ZakatErrorCode::ConfigMissing,
                    reason_key: "error-gold-price-required".to_string(),
                    source_label: Some("ZakatConfig validation".to_string()),
                    suggestion: Some("Run with --gold-price X or set ZAKAT_GOLD_PRICE env var.".to_string()),
                    ..Default::default()
                })));
            }
            if self.silver_price_per_gram <= Decimal::ZERO {
                return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                    code: crate::types::ZakatErrorCode::ConfigMissing,
                    reason_key: "error-silver-price-required".to_string(),
                    source_label: Some("ZakatConfig validation".to_string()),
                    suggestion: Some("Run with --silver-price X or set ZAKAT_SILVER_PRICE env var.".to_string()),
                    ..Default::default()
                })));
            }
        }

        Ok(())
    }

    /// Attempts to load configuration from environment variables.
    #[instrument]
    pub fn from_env() -> Result<Self, ZakatError> {
        debug!("Loading configuration from environment variables");
        let gold_str = env::var("ZAKAT_GOLD_PRICE")
            .map_err(|_| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigMissing,
                reason_key: "error-env-var-missing".to_string(),
                args: Some(std::collections::HashMap::from([("name".to_string(), "ZAKAT_GOLD_PRICE".to_string())])),
                suggestion: Some("Set ZAKAT_GOLD_PRICE environment variable.".to_string()),
                ..Default::default()
            })))?;
        let silver_str = env::var("ZAKAT_SILVER_PRICE")
            .map_err(|_| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigMissing,
                reason_key: "error-env-var-missing".to_string(),
                args: Some(std::collections::HashMap::from([("name".to_string(), "ZAKAT_SILVER_PRICE".to_string())])),
                suggestion: Some("Set ZAKAT_SILVER_PRICE environment variable.".to_string()),
                ..Default::default()
            })))?;

        let gold_price = gold_str.trim().parse::<Decimal>()
            .map_err(|_e| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::InvalidInput,
                reason_key: "error-env-var-invalid".to_string(),
                args: Some(std::collections::HashMap::from([("name".to_string(), "ZAKAT_GOLD_PRICE".to_string())])),
                suggestion: Some("Ensure ZAKAT_GOLD_PRICE is a valid decimal number.".to_string()),
                ..Default::default()
            })))?;
        let silver_price = silver_str.trim().parse::<Decimal>()
            .map_err(|_e| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::InvalidInput,
                reason_key: "error-env-var-invalid".to_string(),
                args: Some(std::collections::HashMap::from([("name".to_string(), "ZAKAT_SILVER_PRICE".to_string())])),
                suggestion: Some("Ensure ZAKAT_SILVER_PRICE is a valid decimal number.".to_string()),
                ..Default::default()
            })))?;

        Ok(Self {
            gold_price_per_gram: gold_price,
            silver_price_per_gram: silver_price,
            ..Default::default()
        })
    }

    /// Attempts to load configuration from a JSON file.
    pub fn try_from_json(path: &str) -> Result<Self, ZakatError> {
        let content = fs::read_to_string(path)
            .map_err(|_e| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-read-file".to_string(),
                suggestion: Some("Ensure the file path is correct and the file exists.".to_string()),
                ..Default::default()
            })))?;
        
        let config: ZakatConfig = serde_json::from_str(&content)
            .map_err(|_e| ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-parse-json".to_string(),
                suggestion: Some("Ensure the JSON is valid and matches the ZakatConfig schema.".to_string()),
                ..Default::default()
            })))?;
            
        config.validate()?;
        Ok(config)
    }

    /// Merges another configuration into this one.
    /// 
    /// Values in `self` take precedence if they are set (non-zero/Some).
    /// If `self` has missing/default values, `other`'s values are used.
    pub fn merge(mut self, other: ZakatConfig) -> Self {
        if self.gold_price_per_gram == Decimal::ZERO {
            self.gold_price_per_gram = other.gold_price_per_gram;
        }
        if self.silver_price_per_gram == Decimal::ZERO {
            self.silver_price_per_gram = other.silver_price_per_gram;
        }
        if self.rice_price_per_kg.is_none() {
            self.rice_price_per_kg = other.rice_price_per_kg;
        }
        if self.rice_price_per_liter.is_none() {
            self.rice_price_per_liter = other.rice_price_per_liter;
        }
        if self.nisab_gold_grams.is_none() {
            self.nisab_gold_grams = other.nisab_gold_grams;
        }
        if self.nisab_silver_grams.is_none() {
            self.nisab_silver_grams = other.nisab_silver_grams;
        }
        if self.nisab_agriculture_kg.is_none() {
            self.nisab_agriculture_kg = other.nisab_agriculture_kg;
        }
        
        self
    }

    // ========== Fluent Helper Methods ========== 

    pub fn with_gold_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.gold_price_per_gram = p;
        }
        self
    }

    pub fn with_silver_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
             self.silver_price_per_gram = p;
        }
        self
    }

    pub fn with_gold_nisab(mut self, grams: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = grams.into_zakat_decimal() {
            self.nisab_gold_grams = Some(p);
        }
        self
    }

    pub fn with_silver_nisab(mut self, grams: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = grams.into_zakat_decimal() {
            self.nisab_silver_grams = Some(p);
        }
        self
    }

    pub fn with_agriculture_nisab(mut self, kg: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = kg.into_zakat_decimal() {
            self.nisab_agriculture_kg = Some(p);
        }
        self
    }

    pub fn with_locale_code(mut self, locale: impl Into<String>) -> Self {
        self.locale_code = locale.into();
        self
    }

    pub fn with_currency_code(mut self, code: impl Into<String>) -> Self {
        self.currency_code = code.into();
        self
    }

    pub fn with_rice_price_per_kg(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.rice_price_per_kg = Some(p);
        }
        self
    }

    pub fn with_rice_price_per_liter(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.rice_price_per_liter = Some(p);
        }
        self
    }

    /// Sets the Zakat strategy using a preset Madhab or custom strategy.
    pub fn with_madhab(mut self, madhab: impl ZakatStrategy + 'static) -> Self {
        let rules = madhab.get_rules();
        self.strategy = Arc::new(madhab);
        self.cash_nisab_standard = rules.nisab_standard;
        self
    }

    /// Sets a custom Zakat strategy from an Arc.
    pub fn with_strategy(mut self, strategy: Arc<dyn ZakatStrategy>) -> Self {
        self.cash_nisab_standard = strategy.get_rules().nisab_standard;
        self.strategy = strategy;
        self
    }

    pub fn with_nisab_standard(mut self, standard: NisabStandard) -> Self {
        self.cash_nisab_standard = standard;
        self
    }

    /// Sets the validation mode (Strict or Permissive).
    /// 
    /// - `Strict`: Enforces all Fiqh validations, errors on missing data.
    /// - `Permissive`: Allows approximations, uses safe defaults for missing data.
    pub fn with_mode(mut self, mode: ZakatMode) -> Self {
        self.mode = mode;
        self
    }

    /// Returns `true` if running in permissive estimation mode.
    pub fn is_permissive(&self) -> bool {
        self.mode == ZakatMode::Permissive
    }

    // Getters
    pub fn get_nisab_gold_grams(&self) -> Decimal {
        self.nisab_gold_grams.unwrap_or(dec!(85))
    }

    pub fn get_nisab_silver_grams(&self) -> Decimal {
        self.nisab_silver_grams.unwrap_or(dec!(595))
    }

    pub fn get_nisab_agriculture_kg(&self) -> Decimal {
        self.nisab_agriculture_kg.unwrap_or(dec!(653))
    }

    pub fn get_monetary_nisab_threshold(&self) -> Decimal {
        let gold_threshold = self.gold_price_per_gram * self.get_nisab_gold_grams();
        let silver_threshold = self.silver_price_per_gram * self.get_nisab_silver_grams();
        
        match self.cash_nisab_standard {
            NisabStandard::Gold => gold_threshold,
            NisabStandard::Silver => silver_threshold,
            NisabStandard::LowerOfTwo => gold_threshold.min(silver_threshold),
        }
    }

    /// Formats a currency amount with the configured locale/currency (basic implementation).
    /// For full i18n support, use `zakat-i18n` crate.
    pub fn format_currency(&self, amount: Decimal) -> String {
        let symbol = match self.currency_code.as_str() {
            "USD" => "$",
            "EUR" => "€",
            "GBP" => "£",
            "IDR" => "Rp",
            "SAR" => "ر.س",
            _ => &self.currency_code,
        };
        format!("{}{:.2}", symbol, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_prices() {
        let config = ZakatConfig::new()
            .with_gold_price(0)
            .with_silver_price(0);
        
        let res = config.validate();
        assert!(res.is_err(), "Validation should fail for zero/default prices");
        
        match res {
            Err(ZakatError::ConfigurationError(details)) => {
                assert!(details.reason_key.contains("error-config-gold-positive"), "Error should match key. Got: {}", details.reason_key);
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_valid_prices() {
        let config = ZakatConfig::test_default();
        let res = config.validate();
        assert!(res.is_ok(), "test_default() should produce valid config");
    }

    #[test]
    fn test_region_presets() {
        // Malaysia (MY) -> Shafi (Shafii) -> Gold Standard
        let config_my = ZakatConfig::for_region("MY");
        assert_eq!(config_my.cash_nisab_standard, NisabStandard::Gold);
        
        // Saudi Arabia (SA) -> Hanbali (Hanafi/Hanbali/Shafi use Gold, but Hanafi uses Silver often)
        // Our implementation for SA uses Gold as a safe baseline for modern GCC.
        let config_sa = ZakatConfig::for_region("SA");
        assert_eq!(config_sa.cash_nisab_standard, NisabStandard::Gold);
    }
}
