//! Internationalization and Localization Module
//!
//! Provides translation and currency formatting capabilities.

use rust_decimal::Decimal;
use rust_embed::RustEmbed;
use fluent_bundle::{FluentResource, FluentArgs};
use fluent_bundle::bundle::FluentBundle;
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use std::str::FromStr;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use icu::locid::Locale;
use icu::decimal::{FixedDecimalFormatter, options::FixedDecimalFormatterOptions};
use fixed_decimal::FixedDecimal;
use writeable::Writeable;

#[derive(RustEmbed)]
#[folder = "assets/locales"]
struct Asset;

/// Supported locales for the Zakat library.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Serialize, Deserialize)]
pub enum ZakatLocale {
    #[default]
    EnUS,
    IdID,
    ArSA,
    /// Dynamic locale loaded at runtime (uses string code).
    Custom,
}

impl ZakatLocale {
    pub fn as_str(&self) -> &'static str {
        match self {
            ZakatLocale::EnUS => "en-US",
            ZakatLocale::IdID => "id-ID",
            ZakatLocale::ArSA => "ar-SA",
            ZakatLocale::Custom => "custom",
        }
    }

    pub fn to_icu_locale(&self) -> Locale {
        self.as_str().parse().expect("Valid BCP-47 locale")
    }

    pub fn currency_code(&self) -> &'static str {
        match self {
            ZakatLocale::EnUS => "USD",
            ZakatLocale::IdID => "IDR",
            ZakatLocale::ArSA => "SAR",
            ZakatLocale::Custom => "USD",
        }
    }
}

impl FromStr for ZakatLocale {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "en-US" | "en" => Ok(ZakatLocale::EnUS),
            "id-ID" | "id" => Ok(ZakatLocale::IdID),
            "ar-SA" | "ar" => Ok(ZakatLocale::ArSA),
            _ => Err(format!("Unsupported locale: {}", s)),
        }
    }
}

/// Trait for formatting currencies.
pub trait CurrencyFormatter {
    fn format_currency(&self, amount: Decimal) -> String;
}

impl CurrencyFormatter for ZakatLocale {
    fn format_currency(&self, amount: Decimal) -> String {
        let locale = self.to_icu_locale();
        
        // Use ICU4X FixedDecimalFormatter with compiled data
        let options = FixedDecimalFormatterOptions::default();
        let formatter = FixedDecimalFormatter::try_new(&locale.into(), options)
            .expect("Failed to create ICU formatter with compiled data");

        // Convert Decimal to FixedDecimal
        let amount_str = amount.to_string();
        let fixed_decimal = FixedDecimal::from_str(&amount_str)
            .unwrap_or_else(|_| FixedDecimal::from(0));

        let formatted_number = formatter.format(&fixed_decimal);
        let number_str = formatted_number.write_to_string().into_owned();

        // Manual fallback for currency symbols
        match self {
            ZakatLocale::EnUS => format!("${}", number_str),
            ZakatLocale::IdID => format!("Rp{}", number_str),
            ZakatLocale::ArSA => format!("{} ر.س", number_str),
            ZakatLocale::Custom => format!("${}", number_str),
        }
    }
}

/// Dynamic locale key for runtime-loaded translations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DynamicLocale(pub String);

impl DynamicLocale {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }
}

/// Translator for multi-locale message translation using Fluent.
/// 
/// Supports both embedded (compile-time) locales and dynamically loaded locales.
#[derive(Clone)]
pub struct Translator {
    /// Embedded locale bundles (loaded from rust-embed assets).
    bundles: std::sync::Arc<HashMap<ZakatLocale, FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>>>,
    /// Dynamically loaded locale bundles (loaded at runtime).
    dynamic_bundles: std::sync::Arc<std::sync::RwLock<HashMap<String, FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>>>>,
}

impl std::fmt::Debug for Translator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dynamic_locales: Vec<String> = self.dynamic_bundles
            .read()
            .map(|guard| guard.keys().cloned().collect())
            .unwrap_or_default();
        
        f.debug_struct("Translator")
         .field("embedded_locales", &self.bundles.keys())
         .field("dynamic_locales", &dynamic_locales)
         .finish()
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

impl Translator {
    pub fn new() -> Self {
        let mut bundles = HashMap::new();
        
        let locales = [
            (ZakatLocale::EnUS, "en-US"),
            (ZakatLocale::IdID, "id-ID"),
            (ZakatLocale::ArSA, "ar-SA"),
        ];

        for (enum_val, code) in locales {
            let lang_id: LanguageIdentifier = code.parse().expect("Parsing lang id failed");
            let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
            
            // Load file content
            let file_path = format!("{}/main.ftl", code);
            if let Some(file) = Asset::get(&file_path) {
                let source = std::str::from_utf8(file.data.as_ref()).expect("Non-utf8 ftl file");
                let resource = FluentResource::try_new(source.to_string())
                    .expect("Failed to parse FTL");
                bundle.add_resource(resource).expect("Failed to add resource");
            } else {
                eprintln!("Warning: Translation file not found for {}", code);
            }
            
            bundles.insert(enum_val, bundle);
        }

        Translator { 
            bundles: std::sync::Arc::new(bundles),
            dynamic_bundles: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }
    
    /// Loads translations from a directory at runtime.
    /// 
    /// Files in the directory should be named with the locale code (e.g., `fr-FR.ftl`, `de-DE.ftl`).
    /// These translations will override embedded translations if the same locale code is used.
    /// 
    /// # Example
    /// ```rust,ignore
    /// let mut translator = Translator::new();
    /// translator.load_from_dir(PathBuf::from("./custom_locales"))?;
    /// 
    /// // Now you can use the loaded locale
    /// let msg = translator.translate_dynamic("fr-FR", "greeting", None);
    /// ```
    /// 
    /// # Errors
    /// Returns an error if the directory cannot be read or contains invalid FTL files.
    pub fn load_from_dir(&self, path: PathBuf) -> Result<Vec<String>, TranslatorError> {
        let mut loaded_locales = Vec::new();
        
        let entries = std::fs::read_dir(&path)
            .map_err(|e| TranslatorError::DirectoryReadError {
                path: path.clone(),
                reason: e.to_string(),
            })?;
        
        for entry in entries {
            let entry = entry.map_err(|e| TranslatorError::DirectoryReadError {
                path: path.clone(),
                reason: e.to_string(),
            })?;
            
            let file_path = entry.path();
            
            // Only process .ftl files
            if file_path.extension().map(|e| e == "ftl").unwrap_or(false) {
                // Extract locale code from filename (e.g., "fr-FR.ftl" -> "fr-FR")
                if let Some(locale_code) = file_path.file_stem().and_then(|s| s.to_str()) {
                    match self.load_ftl_file(&file_path, locale_code) {
                        Ok(()) => {
                            loaded_locales.push(locale_code.to_string());
                            tracing::info!("Loaded dynamic locale: {}", locale_code);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load locale from {:?}: {}", file_path, e);
                        }
                    }
                }
            }
        }
        
        Ok(loaded_locales)
    }
    
    /// Loads a single FTL file for a specific locale.
    /// 
    /// # Example
    /// ```rust,ignore
    /// translator.load_ftl_file(PathBuf::from("./fr-FR.ftl"), "fr-FR")?;
    /// ```
    pub fn load_ftl_file(&self, path: &PathBuf, locale_code: &str) -> Result<(), TranslatorError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TranslatorError::FileReadError {
                path: path.clone(),
                reason: e.to_string(),
            })?;
        
        self.load_ftl_content(locale_code, &content)
    }
    
    /// Loads FTL content directly for a specific locale.
    /// 
    /// Useful for loading translations from strings (e.g., from a database or API).
    /// 
    /// # Example
    /// ```rust,ignore
    /// let ftl_content = r#"
    /// greeting = Bonjour!
    /// farewell = Au revoir!
    /// "#;
    /// translator.load_ftl_content("fr-FR", ftl_content)?;
    /// ```
    pub fn load_ftl_content(&self, locale_code: &str, content: &str) -> Result<(), TranslatorError> {
        let lang_id: LanguageIdentifier = locale_code.parse()
            .map_err(|_| TranslatorError::InvalidLocaleCode(locale_code.to_string()))?;
        
        let resource = FluentResource::try_new(content.to_string())
            .map_err(|(_res, errors)| TranslatorError::ParseError {
                locale: locale_code.to_string(),
                errors: errors.iter().map(|e| e.to_string()).collect(),
            })?;
        
        let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
        bundle.add_resource(resource)
            .map_err(|errors| TranslatorError::ResourceAddError {
                locale: locale_code.to_string(),
                errors: errors.iter().map(|e| e.to_string()).collect(),
            })?;
        
        // Insert into dynamic bundles
        let mut guard = self.dynamic_bundles.write()
            .map_err(|_| TranslatorError::LockError)?;
        guard.insert(locale_code.to_string(), bundle);
        
        Ok(())
    }
    
    /// Lists all available locales (both embedded and dynamic).
    pub fn available_locales(&self) -> Vec<String> {
        let mut locales: Vec<String> = self.bundles.keys()
            .map(|l| l.as_str().to_string())
            .collect();
        
        if let Ok(guard) = self.dynamic_bundles.read() {
            locales.extend(guard.keys().cloned());
        }
        
        locales.sort();
        locales.dedup();
        locales
    }

    /// Translate a key with optional arguments.
    #[allow(clippy::collapsible_if)]
    pub fn translate(&self, locale: ZakatLocale, key: &str, args: Option<&FluentArgs>) -> String {
        let bundle_opt = self.bundles.get(&locale).or_else(|| self.bundles.get(&ZakatLocale::EnUS));
        
        if let Some(bundle) = bundle_opt {
            if let Some(pattern) = bundle.get_message(key).and_then(|msg| msg.value()) {
                let mut errors = vec![];
                let value = bundle.format_pattern(pattern, args, &mut errors);
                return value.to_string();
            }
        }
        
        if locale != ZakatLocale::EnUS {
             return self.translate(ZakatLocale::EnUS, key, args);
        }

        format!("MISSING:{}", key)
    }
    
    /// Translate a key using a dynamic (runtime-loaded) locale.
    /// 
    /// Falls back to embedded en-US if the dynamic locale is not found.
    pub fn translate_dynamic(&self, locale_code: &str, key: &str, args: Option<&FluentArgs>) -> String {
        // First, try dynamic bundles
        if let Ok(guard) = self.dynamic_bundles.read() {
            if let Some(bundle) = guard.get(locale_code) {
                if let Some(pattern) = bundle.get_message(key).and_then(|msg| msg.value()) {
                    let mut errors = vec![];
                    let value = bundle.format_pattern(pattern, args, &mut errors);
                    return value.to_string();
                }
            }
        }
        
        // Fallback: try to parse as ZakatLocale
        if let Ok(embedded_locale) = ZakatLocale::from_str(locale_code) {
            return self.translate(embedded_locale, key, args);
        }
        
        // Final fallback: en-US
        self.translate(ZakatLocale::EnUS, key, args)
    }

    /// Translate with a HashMap of arguments.
    pub fn translate_with_args(&self, locale: ZakatLocale, key: &str, args: Option<&HashMap<String, String>>) -> String {
        if let Some(map) = args {
            let mut f_args = FluentArgs::new();
            for (k, v) in map {
                f_args.set(k.as_str(), v.to_string());
            }
            self.translate(locale, key, Some(&f_args))
        } else {
            self.translate(locale, key, None)
        }
    }

    /// Alias for translate_with_args.
    pub fn translate_map(&self, locale: ZakatLocale, key: &str, args: Option<&HashMap<String, String>>) -> String {
        self.translate_with_args(locale, key, args)
    }
}

/// Errors that can occur when loading translations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TranslatorError {
    #[error("Failed to read directory '{path}': {reason}")]
    DirectoryReadError {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Failed to read file '{path}': {reason}")]
    FileReadError {
        path: PathBuf,
        reason: String,
    },
    
    #[error("Invalid locale code: {0}")]
    InvalidLocaleCode(String),
    
    #[error("Failed to parse FTL for locale '{locale}': {errors:?}")]
    ParseError {
        locale: String,
        errors: Vec<String>,
    },
    
    #[error("Failed to add resource for locale '{locale}': {errors:?}")]
    ResourceAddError {
        locale: String,
        errors: Vec<String>,
    },
    
    #[error("Failed to acquire lock on dynamic bundles")]
    LockError,
}

/// Creates a default Translator instance.
pub fn default_translator() -> Translator {
    Translator::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_currency_formatting() {
        let amount = dec!(1234.56);

        // Test EnUS
        let us = ZakatLocale::EnUS;
        let res_us = us.format_currency(amount);
        println!("EnUS: {}", res_us);
        assert!(res_us.contains("$") || res_us.contains("US$"));
        assert!(res_us.contains("1,234.56"));

        // Test IdID
        let id = ZakatLocale::IdID;
        let res_id = id.format_currency(amount);
        println!("IdID: {}", res_id);
        assert!(res_id.contains("Rp"));
        assert!(res_id.contains("1.234,56"));

        // Test ArSA
        let ar = ZakatLocale::ArSA;
        let res_ar = ar.format_currency(amount);
        println!("ArSA: {}", res_ar);
        assert!(res_ar.contains("١"));
        assert!(res_ar.contains("ر.س") || res_ar.contains("SAR"));
    }
    
    #[test]
    fn test_load_ftl_content_dynamic() {
        let translator = Translator::new();
        
        let french_ftl = r#"
greeting = Bonjour!
farewell = Au revoir!
zakat-due = Zakat dû: { $amount }
"#;
        
        // Load French translations dynamically
        translator.load_ftl_content("fr-FR", french_ftl).unwrap();
        
        // Verify the locale was loaded
        let locales = translator.available_locales();
        assert!(locales.contains(&"fr-FR".to_string()));
        
        // Test translation
        let greeting = translator.translate_dynamic("fr-FR", "greeting", None);
        assert_eq!(greeting, "Bonjour!");
        
        let farewell = translator.translate_dynamic("fr-FR", "farewell", None);
        assert_eq!(farewell, "Au revoir!");
    }
    
    #[test]
    fn test_dynamic_translation_with_args() {
        let translator = Translator::new();
        
        let german_ftl = r#"
welcome = Willkommen, { $name }!
balance = Ihr Guthaben beträgt: { $amount }
"#;
        
        translator.load_ftl_content("de-DE", german_ftl).unwrap();
        
        let mut args = FluentArgs::new();
        args.set("name", "Hans");
        
        let welcome = translator.translate_dynamic("de-DE", "welcome", Some(&args));
        assert!(welcome.contains("Hans"));
        assert!(welcome.contains("Willkommen"));
    }
    
    #[test]
    fn test_dynamic_fallback_to_embedded() {
        let translator = Translator::new();
        
        // Request translation from unknown dynamic locale
        // Should fallback to en-US embedded translations
        let result = translator.translate_dynamic("xx-XX", "unknown-key", None);
        
        // Should return MISSING: since key doesn't exist
        assert!(result.starts_with("MISSING:"));
    }
    
    #[test]
    fn test_available_locales() {
        let translator = Translator::new();
        
        let locales = translator.available_locales();
        
        // Should contain embedded locales
        assert!(locales.contains(&"en-US".to_string()));
        assert!(locales.contains(&"id-ID".to_string()));
        assert!(locales.contains(&"ar-SA".to_string()));
    }
    
    #[test]
    fn test_invalid_locale_code() {
        let translator = Translator::new();
        
        let result = translator.load_ftl_content("not-a-valid-locale!!!", "greeting = Hello");
        
        assert!(result.is_err());
        assert!(matches!(result, Err(TranslatorError::InvalidLocaleCode(_))));
    }
}
