# Javascript / WASM Usage Guide 📦

Using `@islamic/zakat` in Node.js or the Browser.

## Installation

```bash
npm install @islamic/zakat
# or
yarn add @islamic/zakat
```

## Basic Usage (Node.js / ES Modules)

```javascript
import { ZakatConfig, ZakatPortfolio, BusinessZakat } from '@islamic/zakat';

// 1. Configure
const config = new ZakatConfig("85.0", "1.0"); // Gold & Silver Prices as strings

// 2. Create Portfolio
const portfolio = new ZakatPortfolio();

// 3. Add Assets
// Cash: 10,000, Inventory: 5,000
const store = new BusinessZakat("10000", "5000"); 
portfolio.add(store);

// 4. Calculate
const result = portfolio.calculate(config);

console.log(`Total Assets: ${result.total_assets}`);
console.log(`Zakat Due: ${result.total_zakat_due}`);
```

## Browser Usage

Ensure your bundler (Vite, Webpack) is configured to load WASM.

```javascript
import init, { ZakatConfig, calculate_portfolio } from '@islamic/zakat';

async function run() {
    await init(); // Initialize WASM
    
    // ... use classes as above
}

run();
```

## Type Mapping (Rust → JavaScript)

Understanding how Rust types are serialized to JavaScript:

| Rust Type | JS Type | Notes |
|:----------|:--------|:------|
| `Decimal` | `string` | Preserves 28-digit precision. Parse with `decimal.js` or `BigInt` for math. |
| `Option<T>` | `T \| null` | `None` becomes `null`. |
| `Uuid` | `string` | Standard UUID format `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`. |
| `Vec<T>` | `Array<T>` | Arrays serialize directly. |
| `bool` | `boolean` | Direct mapping. |
| `HashMap<K,V>` | `object` | Keys become object properties. |

> **Why `string` for Decimal?**
> JavaScript `number` (IEEE 754 float) loses precision after ~15 digits. Zakat calculations require financial precision, so we serialize as `string` to preserve all 28 decimal digits.

## Precision Handling

For arithmetic operations on Zakat amounts, use a precision library:

### Using decimal.js

```javascript
import { Decimal } from 'decimal.js';

const result = await calculate_portfolio(config, assets);

// Parse string-based Decimal values
const zakatDue = new Decimal(result.total_zakat_due);
const totalAssets = new Decimal(result.total_assets);

// Perform precise arithmetic
const percentage = zakatDue.div(totalAssets).mul(100);
console.log(`Zakat is ${percentage.toFixed(4)}% of assets`);
```

### Using BigInt (for whole units)

```javascript
// If working with whole currency units (e.g., cents)
const zakatCents = BigInt(result.zakat_due.replace('.', ''));
```

## Error Handling

The WASM module throws structured errors with rich metadata:

```javascript
try {
    const result = await calculate_single_asset(config, asset);
} catch (error) {
    // Error is a structured object:
    // {
    //   code: "INVALID_INPUT",
    //   message: "Cash value must be non-negative",
    //   field: "cash",
    //   hint: "Enter a positive number or zero",
    //   source_label: "Main Store"
    // }
    
    console.error(`[${error.code}] ${error.message}`);
    if (error.field) {
        highlightField(error.field);
    }
    if (error.hint) {
        showTooltip(error.hint);
    }
}
```

## Debugging

### Enable Panic Hooks

Call `init_hooks()` immediately after WASM initialization for better error traces:

```javascript
import init, { init_hooks, calculate_portfolio } from '@islamic/zakat';

async function run() {
    await init();
    init_hooks(); // Enables console_error_panic_hook for better stack traces
    
    // Now panics will show Rust file/line info in browser console
}
```

### Verbose Error Mode

Enable tracing in development:

```javascript
// In development, errors include source_label to identify which asset failed
const result = await calculate_portfolio(config, assets);

for (const item of result.results) {
    if (item.type === 'Failure') {
        console.error(`Asset '${item.source}' failed: ${item.error}`);
    }
}
```

