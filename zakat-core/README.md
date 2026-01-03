# zakat-core

Core mathematical logic, data structures, Fiqh rules, and input validation for Zakat calculations.

## Overview

`zakat-core` provides the foundational building blocks for Zakat calculation:

- Type-safe monetary calculations using `rust_decimal`
- Input sanitization supporting international number formats
- Fiqh-compliant calculation rules for all asset types
- Madhab-aware strategy pattern for different schools of thought

## Asset Types

| Module | Description |
|--------|-------------|
| `maal::precious_metals` | Gold and Silver with purity and weight calculations |
| `maal::business` | Trade goods, inventory, receivables, and liabilities |
| `maal::agriculture` | Crops with irrigation method differentiation |
| `maal::livestock` | Camels, cattle, and sheep with Nisab tables |
| `maal::income` | Professional income (gross/net methods) |
| `maal::investments` | Stocks, bonds, and cryptocurrency |
| `maal::mining` | Minerals and buried treasure (Rikaz) |

## Usage

```rust
use zakat_core::prelude::*;
use zakat_core::maal::precious_metals::PreciousMetals;
use zakat_core::config::ZakatConfig;

let config = ZakatConfig::default();

let gold = PreciousMetals::gold(100.0)
    .purity(24)
    .calculate_zakat(&config)?;

println!("Zakat due: {}", gold.zakat_due);
```

## Input Sanitization

The crate handles international number formats automatically:

```rust
use zakat_core::inputs::IntoZakatDecimal;

// US format
let amount: Decimal = "$1,234.56".into_zakat_decimal()?;

// European format
let amount: Decimal = "1.234,56".into_zakat_decimal()?;

// Arabic numerals
let amount: Decimal = "١٢٣٤.٥٦".into_zakat_decimal()?;
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `async` | Async calculation support with Tokio |
| `wasm` | WebAssembly bindings |
| `python` | Python bindings via PyO3 |
| `uniffi` | Kotlin/Swift bindings via UniFFI |

## License

MIT
