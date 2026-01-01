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
