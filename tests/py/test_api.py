#!/usr/bin/env python
"""Quick API test for zakatrs."""

import zakatrs

# Test creating PreciousMetals
gold = zakatrs.PreciousMetals(
    weight_grams="100",
    metal_type="Gold",
    purity="999",
    usage="Investment"
)
print("Gold created:", gold)
print("  weight_grams:", gold.weight_grams)
print("  metal_type:", gold.metal_type)

# Create config
config = zakatrs.ZakatConfig(gold_price="65.00", silver_price="0.75")
print("Config created:", config)
print("  gold_price_per_gram:", config.gold_price_per_gram)

# Calculate
result = gold.calculate(config)
print("Result:", result)
print("  is_payable:", result.is_payable)
print("  zakat_due:", result.zakat_due)
print("  net_assets:", result.net_assets)
