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

# Zakat

[![Crates.io](https://img.shields.io/crates/v/zakat.svg)](https://crates.io/crates/zakat)
[![Docs.rs](https://docs.rs/zakat/badge.svg)](https://docs.rs/zakat)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust library for Islamic Zakat calculation. Uses `rust_decimal` for precision.

## Features

- Gold, Silver, Business, Agriculture, Livestock, Mining & Rikaz
- Stocks, Mutual Funds, Crypto (as liquid assets)
- Professional Income (Gross/Net)
- Zakat Fitrah
- Configurable Nisab thresholds
- Portfolio aggregation

## Install

```toml
[dependencies]
zakat = "0.1.3"
rust_decimal = "1.39"
rust_decimal_macros = "1.39"
```

## Usage

### Business Zakat

```rust
use zakat::{ZakatConfig, CalculateZakat};
use zakat::maal::business::{BusinessAssets, BusinessZakatCalculator};

fn main() {
    let config = ZakatConfig::new(65, 1); // gold $65/g, silver $1/g

    let assets = BusinessAssets::new(
        50000, // cash
        20000, // inventory
        5000,  // receivables
        1000   // debt
    );

    let calc = BusinessZakatCalculator::new(assets, &config).unwrap();
    let result = calc.calculate_zakat(None).unwrap();

    if result.is_payable {
        println!("Zakat: ${}", result.zakat_due);
    }
}
```

### Portfolio

```rust
use zakat::{ZakatConfig, ZakatPortfolio, WealthType};
use zakat::maal::precious_metals::PreciousMetal;
use zakat::maal::investments::{InvestmentAssets, InvestmentType};
use zakat::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};
use rust_decimal_macros::dec;

fn main() {
    let config = ZakatConfig::new(65, 1);

    let portfolio = ZakatPortfolio::new()
        .add_calculator(IncomeZakatCalculator::new(
            5000, 0, IncomeCalculationMethod::Gross, &config
        ).unwrap())
        .add_calculator(PreciousMetal::new(
            100, WealthType::Gold, &config
        ).unwrap())
        .add_calculator_with_debt(InvestmentAssets::new(
            20000, InvestmentType::Crypto, &config
        ).unwrap(), dec!(2000.0));

    let result = portfolio.calculate_total().unwrap();
    println!("Total: ${}", result.total_zakat_due);
}
```

### Custom Nisab

```rust
use zakat::ZakatConfig;

let config = ZakatConfig::new(65, 1)
    .with_gold_nisab(87)
    .with_agriculture_nisab(700);
```

## Modules

| Module | Nisab |
| :--- | :--- |
| `maal::precious_metals` | 85g Gold / 595g Silver |
| `maal::business` | 85g Gold |
| `maal::income` | 85g Gold |
| `maal::investments` | 85g Gold |
| `maal::agriculture` | 653 kg |
| `maal::livestock` | Count-based |
| `maal::mining` | Rikaz: None / Mines: 85g Gold |
| `fitrah` | N/A |

## Contributing

1. Add tests
2. Use `rust_decimal`
3. Run `cargo test`

## Support

<div align="center">

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-%E2%9D%A4-ea4aaa?style=for-the-badge&logo=github-sponsors)](https://github.com/sponsors/IRedDragonICY)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/ireddragonicy)
[![Patreon](https://img.shields.io/badge/Patreon-F96854?style=for-the-badge&logo=patreon&logoColor=white)](https://patreon.com/ireddragonicy)
[![PayPal](https://img.shields.io/badge/PayPal-00457C?style=for-the-badge&logo=paypal&logoColor=white)](https://paypal.com/paypalme/IRedDragonICY)
[![Saweria](https://img.shields.io/badge/Saweria-F4801C?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSIjZmZmZmZmIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjEyIiByPSI4Ii8+PC9zdmc+)](https://saweria.co/IRedDragonICY)

</div>

> *"Those who spend their wealth in the cause of Allah..."* — **Al-Baqarah 2:262**

## License

MIT
