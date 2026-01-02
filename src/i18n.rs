use rust_decimal::Decimal;
use rust_embed::RustEmbed;
use fluent_bundle::{FluentResource, FluentArgs};
use fluent_bundle::bundle::FluentBundle;
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use std::str::FromStr;

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
}

impl ZakatLocale {
    pub fn as_str(&self) -> &'static str {
        match self {
            ZakatLocale::EnUS => "en-US",
            ZakatLocale::IdID => "id-ID",
            ZakatLocale::ArSA => "ar-SA",
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

/// Trait for formatting usage.
pub trait CurrencyFormatter {
    fn format_currency(&self, amount: Decimal) -> String;
}

impl CurrencyFormatter for ZakatLocale {
    fn format_currency(&self, amount: Decimal) -> String {
        let locale = self.to_icu_locale();
        
        // Use ICU4X FixedDecimalFormatter with compiled data
        // TODO: Use icu::experimental::currency::CurrencyFormatter when available in crates.io
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
            ZakatLocale::IdID => format!("Rp{}", number_str), // Often Rp 1.234, but standard without space is sometimes used, keeping consistent
            ZakatLocale::ArSA => format!("{} ر.س", number_str),
        }
    }
}

#[derive(Clone)]
pub struct Translator {
    bundles: std::sync::Arc<HashMap<ZakatLocale, FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>>>,
}

impl std::fmt::Debug for Translator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Translator")
         .field("locales", &self.bundles.keys())
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

        Translator { bundles: std::sync::Arc::new(bundles) }
    }

    #[allow(clippy::collapsible_if)]
    pub fn translate(&self, locale: ZakatLocale, key: &str, args: Option<&FluentArgs>) -> String {
        let bundle_opt: Option<&FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>> = self.bundles.get(&locale).or_else(|| self.bundles.get(&ZakatLocale::EnUS));
        
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

    pub fn translate_map(&self, locale: ZakatLocale, key: &str, args: Option<&HashMap<String, String>>) -> String {
        self.translate_with_args(locale, key, args)
    }
}

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
        // Expect: "Rp 1.234,56" or similar
        assert!(res_id.contains("Rp"));
        // Note: ICU might use non-breaking space or similar, so we check for components
        assert!(res_id.contains("1.234,56"));

        // Test ArSA
        let ar = ZakatLocale::ArSA;
        let res_ar = ar.format_currency(amount);
        println!("ArSA: {}", res_ar);
        // 1234.56 -> ١٬٢٣٤٫٥٦
        assert!(res_ar.contains("١"));
        assert!(res_ar.contains("ر.س") || res_ar.contains("SAR"));
    }
}

