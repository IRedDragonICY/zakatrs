# zakat-providers

Live price fetching providers for Zakat calculations.

## Overview

`zakat-providers` enables real-time metal pricing for accurate Nisab calculations:

- Async price fetching with caching
- Multiple provider support (Binance, custom)
- WASM-compatible implementation
- Configurable cache TTL

## Platform Support

| Platform | HTTP Client |
|----------|-------------|
| Native (Linux, macOS, Windows) | `reqwest` |
| WebAssembly | `gloo-net` |

## Usage

```rust
use zakat_providers::{PriceProvider, BinancePriceProvider, CachedPriceProvider};

// Create provider with 5-minute cache
let provider = CachedPriceProvider::new(
    BinancePriceProvider::default(),
    300, // TTL in seconds
);

// Fetch current prices
let prices = provider.get_prices().await?;
println!("Gold: ${}/gram", prices.gold_per_gram);
println!("Silver: ${}/gram", prices.silver_per_gram);
```

## Static Provider

For testing or offline use:

```rust
use zakat_providers::{StaticPriceProvider, PriceProvider};

let provider = StaticPriceProvider::new(65.0, 0.80)?;
let prices = provider.get_prices().await?;
```

## Custom Providers

Implement the `PriceProvider` trait:

```rust
use zakat_providers::{PriceProvider, Prices};
use async_trait::async_trait;

struct MyProvider;

#[async_trait]
impl PriceProvider for MyProvider {
    async fn get_prices(&self) -> Result<Prices, ZakatError> {
        // Fetch from your data source
        Ok(Prices::new(65.0, 0.80)?)
    }
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `live-pricing` | Enable live API calls (default) |

## Network Configuration

```rust
use zakat_providers::{NetworkConfig, BinancePriceProvider};

let config = NetworkConfig {
    timeout_seconds: 30,
    ..Default::default()
};

let provider = BinancePriceProvider::new(&config);
```

## License

MIT
