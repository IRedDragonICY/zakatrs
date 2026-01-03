# zakat-i18n

Internationalization and localization support for Zakat calculations.

## Overview

`zakat-i18n` provides:

- Fluent-based translation system
- ICU4X currency formatting with locale-aware symbols
- Embedded locale files for zero-configuration deployment

## Supported Locales

| Locale | Language |
|--------|----------|
| `en-US` | English (United States) |
| `id-ID` | Indonesian |
| `ar-SA` | Arabic (Saudi Arabia) |

## Usage

```rust
use zakat_i18n::{Translator, ZakatLocale, format_currency};
use rust_decimal_macros::dec;

// Create translator
let translator = Translator::new(ZakatLocale::Indonesian);

// Format currency
let formatted = format_currency(dec!(1234567.89), "IDR", ZakatLocale::Indonesian);
// Output: "Rp 1.234.567,89"
```

## Translation Keys

The library uses structured translation keys:

```fluent
# en-US/main.ftl
zakat-gold-calculation = Gold Zakat: {$weight}g at {$purity}K purity
nisab-threshold = Nisab threshold: {$amount}
```

## Adding New Locales

1. Create a new directory under `assets/locales/{locale-code}/`
2. Add `main.ftl` with translations
3. Rebuild the crate (locales are embedded at compile time)

## Dependencies

- `fluent` - Mozilla's localization system
- `icu` - ICU4X for currency and number formatting
- `rust-embed` - Compile-time locale embedding

## License

MIT
