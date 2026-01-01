use rust_decimal::Decimal;
use rust_embed::RustEmbed;
use fluent_bundle::{FluentResource, FluentArgs};
use fluent_bundle::bundle::FluentBundle;
use unic_langid::LanguageIdentifier;
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::str::FromStr;

use serde::{Serialize, Deserialize};

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
        use rust_decimal::RoundingStrategy;
        
        // Round to 2 decimals first
        let rounded = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
        let s = rounded.to_string();
        
        let (int_part, frac_part) = match s.split_once('.') {
            Some((i, f)) => (i, Some(f)),
            None => (s.as_str(), None),
        };

        // Helper to insert separators
        let format_number = |num_str: &str, sep: char| -> String {
             let mut result = String::new();
             let chars: Vec<char> = num_str.chars().rev().collect();
             for (i, c) in chars.iter().enumerate() {
                 if i > 0 && i % 3 == 0 && *c != '-' {
                     result.push(sep);
                 }
                 result.push(*c);
             }
             result.chars().rev().collect()
        };

        match self {
            ZakatLocale::EnUS => {
                let main = format_number(int_part, ',');
                if let Some(f) = frac_part {
                    format!("${}.{}", main, f)
                } else {
                    format!("${}", main)
                }
            },
            ZakatLocale::IdID => {
                // Indonesia: thousands separator '.', decimal ','
                // Example: Rp1.234,56
                let main = format_number(int_part, '.');
                if let Some(f) = frac_part {
                    format!("Rp{},{}", main, f)
                } else {
                    format!("Rp{}", main)
                }
            },
            ZakatLocale::ArSA => {
                // Arabic: Uses standard numerals usually in finance but let's stick to locale specific if requested.
                // Request says: "keep ASCII for now to minimize complexity, but respect locale formatting"
                // Saudi: usually SAR prefix/suffix. Let's use suffix " ر.س" or prefix "SAR ".
                // Common convention: 1,234.56 SAR
                let main = format_number(int_part, ',');
                if let Some(f) = frac_part {
                    format!("{}.{} ر.س", main, f) 
                } else {
                    format!("{} ر.س", main)
                }
            }
        }
    }
}

pub struct Translator {
    bundles: HashMap<ZakatLocale, FluentBundle<FluentResource, intl_memoizer::concurrent::IntlLangMemoizer>>,
}

impl Translator {
    fn new() -> Self {
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

        Translator { bundles }
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
}

pub static TRANSLATOR: Lazy<Translator> = Lazy::new(Translator::new);
