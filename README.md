<div align="center">
<h1>السلام عليكم</h1>
</div>

```text
███████╗ █████╗ ██╗  ██╗ █████╗ ████████╗
╚══███╔╝██╔══██╗██║ ██╔╝██╔══██╗╚══██╔══╝
  ███╔╝ ███████║█████╔╝ ███████║   ██║   
 ███╔╝  ██╔══██║██╔═██╗ ██╔══██║   ██║   
███████╗██║  ██║██║  ██╗██║  ██║   ██║   
╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   
```

# Zakat: The Definitive Islamic Alms Calculation Library

[![Crates.io](https://img.shields.io/crates/v/zakat.svg)](https://crates.io/crates/zakat)
[![Docs.rs](https://docs.rs/zakat/badge.svg)](https://docs.rs/zakat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Zakat** is a production-grade, type-safe Rust library designed to bridge **Classical Fiqh** with **Modern Finance**. It provides mathematically precise calculations for essentially all major wealth types, including cryptocurrencies, stocks, livestock, and professional income, while handling currency precision flawlessly using `rust_decimal`.

---

## Features

-   **Type-Safe Precision**: Zero floating-point errors. All calculations use `rust_decimal`.
-   **Comprehensive Coverage**:
    -   **Maal (Wealth)**: Gold, Silver, Business/Trade, Agriculture, Livestock (Camels, Cows, Sheep), Mining & Rikaz.
    -   **Modern Assets**: Stocks, Mutual Funds, and Cryptocurrencies (treated as liquid assets).
    -   **Income**: Professional Income calculation (Gross & Net support).
    -   **Fitrah**: Per-capita food staple calculation.
-   **Fiqh Compliant & Flexible**:
    -   Built-in default Nisab thresholds (e.g., 85g Gold).
    -   **Fully Configurable**: Override thresholds to match specific Fatwa or regional standards.
    -   **Debt Deduction**: Flexible logic to deduct liabilities before or after Nisab checks as per config.
-   **Portfolio Management**: Builder pattern to aggregate diverse assets and calculate total Zakat due in one go.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
zakat = "0.1.0"
rust_decimal = "1.39"
rust_decimal_macros = "1.39"
```

## Quick Start

### 1. Simple Business Calculation

```rust
use zakat::{ZakatConfig, CalculateZakat};
use zakat::maal::business::{BusinessAssets, BusinessZakatCalculator};
use rust_decimal_macros::dec;

fn main() {
    // 1. Configure Prices (e.g., Gold = $65/gram)
    let config = ZakatConfig::new(dec!(65.0), dec!(1.0));

    // 2. Define Assets
    let assets = BusinessAssets::new(
        dec!(50000.0), // Cash on Hand
        dec!(20000.0), // Inventory Value
        dec!(5000.0),  // Receivables
        dec!(1000.0)   // Short Term Debt
    );

    // 3. Calculate
    let calculator = BusinessZakatCalculator::new(assets, &config).unwrap();
    let result = calculator.calculate_zakat(None).unwrap();

    if result.is_payable {
        println!("Zakat Due: ${}", result.zakat_due);
    } else {
        println!("Nisab not reached.");
    }
}
```

### 2. Portfolio Management (The "Enterprise" Way)

Handling a complex user scenario (e.g., "Mr. Ahmad") who has income, gold, investments, and debts.

```rust
use zakat::{ZakatConfig, ZakatPortfolio, WealthType};
use zakat::maal::precious_metals::PreciousMetal;
use zakat::maal::investments::{InvestmentAssets, InvestmentType};
use zakat::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};
use rust_decimal_macros::dec;

fn main() {
    // Global Config
    let config = ZakatConfig::new(dec!(65.0), dec!(1.0));

    // Initialize Portfolio
    let portfolio = ZakatPortfolio::new()
        // Add Monthly Income (Gross Method)
        .add_calculator(IncomeZakatCalculator::new(
            dec!(5000.0), 
            dec!(0.0), 
            IncomeCalculationMethod::Gross, 
            &config
        ).unwrap())
        // Add Gold Stash
        .add_calculator(PreciousMetal::new(
            dec!(100.0), // 100 grams
            WealthType::Gold, 
            &config
        ).unwrap())
        // Add Crypto Portfolio with Debt Deduction
        .add_calculator_with_debt(InvestmentAssets::new(
            dec!(20000.0), 
            InvestmentType::Crypto, 
            &config
        ).unwrap(), dec!(2000.0)); // Deduct $2k personal debt

    // Execute
    let result = portfolio.calculate_total().unwrap();

    println!("Total Zakat Due: ${}", result.total_zakat_due);
    // Output breakdown...
}
```

## Advanced Configuration (Custom Nisab)

Different regions or scholar councils may have different standards for Nisab. You can override defaults easily:

```rust
use zakat::ZakatConfig;
use rust_decimal_macros::dec;

let mut config = ZakatConfig::new(dec!(65.0), dec!(1.0));

// Override Gold Nisab to 87g (some opinions) instead of default 85g
config.nisab_gold_grams = Some(dec!(87.0));

// Override Agriculture Nisab
config.nisab_agriculture_kg = Some(dec!(700.0));
```

## Supported Modules

| Module | Features | Nisab Basis |
| :--- | :--- | :--- |
| `maal::precious_metals` | Gold, Silver | 85g Gold / 595g Silver |
| `maal::business` | Cash, Inventory, Receivables | 85g Gold Equiv |
| `maal::income` | Professional Income (Gross/Net) | 85g Gold Equiv |
| `maal::investments` | Stocks, Crypto, Mutual Funds | 85g Gold Equiv |
| `maal::agriculture` | Rain (10%), Irrigated (5%), Mixed | 5 Wasq (~653 kg) |
| `maal::livestock` | Camels, Cows, Sheep (Tiered logic) | Count-based (e.g. 40 Sheep)|
| `maal::mining` | Rikaz (20% flat), Mines (2.5%) | None (Rikaz) / 85g Gold |
| `fitrah` | Per person food staple | N/A |

## Contributing

Contributions are welcome! Please ensure you:
1.  Add unit tests for any new logic.
2.  Maintain `rust_decimal` usage for precision.
3.  Run `cargo test` before submitting.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
