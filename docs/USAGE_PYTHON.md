# Python Usage Guide 🐍

Using `zakatrs` from Python.

## Installation

```bash
pip install zakatrs
```

## Basic Usage

```python
from zakatrs import ZakatConfig, ZakatPortfolio, BusinessZakat, PreciousMetals, IncomeZakatCalculator, InvestmentAssets

# 1. Configure Zakat (Gold: $85/g, Silver: $1.0/g)
# Note: Input prices as strings to preserve decimal precision
config = ZakatConfig(gold_price="85.0", silver_price="1.0")

# 2. Create Portfolio
portfolio = ZakatPortfolio()

# 3. Add Assets

# Business: Cash $10k, Merchandise $5k
biz = BusinessZakat(cash="10000", merchandise="5000", receivables="0", liabilities="0")
portfolio.add(biz)

# Precious Metals: 100g Gold
gold = PreciousMetals(weight="100", metal_type="gold")
portfolio.add(gold)

# Investments: Crypto worth $20k
crypto = InvestmentAssets(value="20000", investment_type="crypto")
portfolio.add(crypto)

# Income: $5000 Salary, Gross Method
salary = IncomeZakatCalculator(income="5000", method="gross")
portfolio.add(salary)

# 4. Calculate
result = portfolio.calculate(config)

print(f"Total Assets: ${result.total_assets}")
print(f"Total Zakat Due: ${result.total_zakat_due}")

# You can also get a dictionary representation
import json
print(json.dumps(result.to_dict(), indent=2))
```

## Precision Handling

The library treats all monetary inputs as high-precision decimals.

```python
# Pass strings for precise monetary values
config = ZakatConfig(gold_price="65.50", silver_price="0.85")
```

## Accessing Details

```python
# Calculate individual asset
details = biz.calculate(config)

if details.is_payable:
    print(f"Net Assets: {details.net_assets}")
    print(f"Zakat Due: {details.zakat_due}")
else:
    print(f"Not payable. Reason: {details.status_reason}")
```
