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

## Type Mapping (Rust → Python)

Understanding how Rust types are serialized to Python:

| Rust Type | Python Type | Notes |
|:----------|:------------|:------|
| `Decimal` | `str` | Preserves 28-digit precision. Convert to `decimal.Decimal` for math. |
| `Option<T>` | `T \| None` | `None` in Rust becomes `None` in Python. |
| `Uuid` | `str` | Standard UUID format. Use `uuid.UUID(s)` to parse. |
| `Vec<T>` | `list[T]` | Lists serialize directly. |
| `bool` | `bool` | Direct mapping. |
| `HashMap<K,V>` | `dict[K,V]` | Dictionaries serialize directly. |

> **Why `str` for Decimal?**
> Python floats lose precision. For financial calculations, the library uses strings to preserve all 28 decimal digits of `rust_decimal::Decimal`.

## Precision Handling

Convert string values to Python's `decimal.Decimal` for precise arithmetic:

```python
from decimal import Decimal, ROUND_DOWN

result = portfolio.calculate(config)

# Convert string-based amounts to Python Decimal
zakat_due = Decimal(result.zakat_due)
total_assets = Decimal(result.total_assets)

# Perform precise arithmetic
percentage = (zakat_due / total_assets * 100).quantize(Decimal('0.01'), rounding=ROUND_DOWN)
print(f"Zakat is {percentage}% of assets")

# Round for display
display_amount = zakat_due.quantize(Decimal('0.01'))
print(f"Pay: ${display_amount}")
```

## Advanced: Using `to_dict()`

The `to_dict()` method returns a complete Python dictionary for serialization:

```python
import json
from decimal import Decimal

result = portfolio.calculate(config)
data = result.to_dict()

# Access nested fields
print(f"Status: {data['status']}")
print(f"Total Zakat: {data['total_zakat_due']}")

# Convert back to high-precision Decimal
for item in data['results']:
    if item['type'] == 'Success':
        zakat = Decimal(item['details']['zakat_due'])
        print(f"  {item['details']['label']}: ${zakat}")

# Serialize to JSON for API response
json_output = json.dumps(data, indent=2)
```

## Error Handling

Errors from the Rust library are raised as Python exceptions:

```python
from zakatrs import ZakatConfig, BusinessZakat, ZakatError

try:
    # Invalid input: negative cash
    biz = BusinessZakat(cash="-1000", merchandise="0")
    result = biz.calculate(config)
except ZakatError as e:
    # Structured error from Rust
    print(f"Code: {e.code}")
    print(f"Message: {e.message}")
    print(f"Field: {e.field}")  # e.g., "cash"
    print(f"Hint: {e.hint}")    # e.g., "Enter a positive number"
except ValueError as e:
    # Parsing error
    print(f"Invalid input: {e}")
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

# Access calculation trace
for step in details.calculation_trace:
    print(f"  {step.description}: {step.amount}")
```

## Debugging

Enable verbose logging to trace calculation issues:

```python
import logging

# Enable Rust library logging (via tracing subscriber)
logging.basicConfig(level=logging.DEBUG)

# The library will emit debug logs for:
# - Configuration validation
# - Nisab threshold calculations
# - Asset-by-asset processing
```

```
