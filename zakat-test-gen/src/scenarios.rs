//! Scenario builder for generating test cases.
//!
//! This module contains the actual logic to create test scenarios
//! by executing the real Rust library.

use crate::schema::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use zakat_core::prelude::*;
use zakat_core::maal::business::BusinessZakat;
use zakat_core::maal::precious_metals::{PreciousMetals, JewelryUsage};

/// Generates all test scenarios by executing the Rust library.
pub fn generate_all_scenarios() -> Vec<TestCase> {
    let mut cases = Vec::new();

    // ========== HAPPY PATH: BUSINESS ==========
    cases.extend(business_happy_path());

    // ========== HAPPY PATH: GOLD ==========
    cases.extend(gold_happy_path());

    // ========== HAPPY PATH: SILVER ==========
    cases.extend(silver_happy_path());

    // ========== EDGE CASES ==========
    cases.extend(edge_cases());

    // ========== PRECISION TESTS ==========
    cases.extend(precision_tests());

    // ========== CONFIGURATION TESTS ==========
    cases.extend(configuration_tests());

    // ========== VALIDATION / ERROR TESTS ==========
    cases.extend(error_tests());

    cases
}

// ============================================================================
// BUSINESS HAPPY PATH
// ============================================================================

fn business_happy_path() -> Vec<TestCase> {
    vec![
        // Business above nisab - basic payable case
        generate_business_case(
            "business_001",
            "Business with cash above nisab - payable",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "10000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Business with all asset types
        generate_business_case(
            "business_002",
            "Business with cash, inventory, and receivables",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "5000")
                .field("inventory_value", "3000")
                .field("receivables", "2000")
                .hawl(true),
        ),

        // Business with liabilities
        generate_business_case(
            "business_003",
            "Business with liabilities deducted",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "15000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .liabilities("5000")
                .hawl(true),
        ),

        // Business below nisab - exempt
        generate_business_case(
            "business_004",
            "Business below nisab - exempt",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "1000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Business exactly at nisab
        generate_business_case(
            "business_005",
            "Business exactly at nisab threshold",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "8500")  // 85g * $100 = $8500
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),
    ]
}

fn generate_business_case(
    id: &str,
    description: &str,
    category: TestCategory,
    config: TestConfig,
    input: TestInput,
) -> TestCase {
    // Build the actual Rust config
    let madhab = match config.madhab.as_str() {
        "shafi" => Madhab::Shafi,
        "maliki" => Madhab::Maliki,
        "hanbali" => Madhab::Hanbali,
        _ => Madhab::Hanafi,
    };

    let gold_price: Decimal = config.gold_price_per_gram.parse().unwrap_or(dec!(0));
    let silver_price: Decimal = config.silver_price_per_gram.parse().unwrap_or(dec!(0));

    let zakat_config = ZakatConfig::new()
        .with_madhab(madhab)
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);

    // Build the business asset
    let cash: Decimal = input.fields.get("cash_on_hand")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .unwrap_or(dec!(0));
    let inventory: Decimal = input.fields.get("inventory_value")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .unwrap_or(dec!(0));
    let receivables: Decimal = input.fields.get("receivables")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .unwrap_or(dec!(0));
    let liabilities: Decimal = input.liabilities_due_now.parse().unwrap_or(dec!(0));

    let business = BusinessZakat::new()
        .cash(cash)
        .inventory(inventory)
        .receivables(receivables)
        .add_liability("Liabilities", liabilities)
        .hawl(input.hawl_satisfied);

    // Execute the calculation
    let expected = match business.calculate_zakat(&zakat_config) {
        Ok(details) => ExpectedResult::success(
            details.is_payable,
            details.zakat_due,
            details.total_assets,
            details.net_assets,
            details.nisab_threshold,
        ),
        Err(e) => ExpectedResult::error(e.code()),
    };

    TestCase {
        id: id.to_string(),
        description: description.to_string(),
        category,
        asset_type: AssetType::Business,
        config,
        input,
        expected,
    }
}

// ============================================================================
// GOLD HAPPY PATH
// ============================================================================

fn gold_happy_path() -> Vec<TestCase> {
    vec![
        // Gold above nisab - payable
        generate_gold_case(
            "gold_001",
            "Gold 100g above 85g nisab - payable",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "24")
                .field("usage", "investment")
                .hawl(true),
        ),

        // Gold below nisab - exempt
        generate_gold_case(
            "gold_002",
            "Gold 80g below 85g nisab - exempt",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "80")
                .field("purity", "24")
                .field("usage", "investment")
                .hawl(true),
        ),

        // Gold with purity adjustment - 18K
        generate_gold_case(
            "gold_003",
            "Gold 100g at 18K purity - effective 75g < nisab",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "18")
                .field("usage", "investment")
                .hawl(true),
        ),

        // Gold with liabilities bringing below nisab
        generate_gold_case(
            "gold_004",
            "Gold 100g with liabilities bringing net below nisab",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "24")
                .field("usage", "investment")
                .liabilities("2000")
                .hawl(true),
        ),

        // Gold exactly at nisab
        generate_gold_case(
            "gold_005",
            "Gold exactly at 85g nisab threshold",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "85")
                .field("purity", "24")
                .field("usage", "investment")
                .hawl(true),
        ),
    ]
}

fn generate_gold_case(
    id: &str,
    description: &str,
    category: TestCategory,
    config: TestConfig,
    input: TestInput,
) -> TestCase {
    let madhab = match config.madhab.as_str() {
        "shafi" => Madhab::Shafi,
        "maliki" => Madhab::Maliki,
        "hanbali" => Madhab::Hanbali,
        _ => Madhab::Hanafi,
    };

    let gold_price: Decimal = config.gold_price_per_gram.parse().unwrap_or(dec!(0));
    let silver_price: Decimal = config.silver_price_per_gram.parse().unwrap_or(dec!(0));

    let zakat_config = ZakatConfig::new()
        .with_madhab(madhab)
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);

    let weight: Decimal = input.fields.get("weight_grams")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .unwrap_or(dec!(0));
    let purity: u32 = input.fields.get("purity")
        .and_then(|v| v.as_str())
        .unwrap_or("24")
        .parse()
        .unwrap_or(24);
    let usage = match input.fields.get("usage").and_then(|v| v.as_str()).unwrap_or("investment") {
        "personal" | "personal_use" => JewelryUsage::PersonalUse,
        _ => JewelryUsage::Investment,
    };
    let liabilities: Decimal = input.liabilities_due_now.parse().unwrap_or(dec!(0));

    let gold = PreciousMetals::gold(weight)
        .purity(purity)
        .usage(usage)
        .add_liability("Liabilities", liabilities)
        .hawl(input.hawl_satisfied);

    let expected = match gold.calculate_zakat(&zakat_config) {
        Ok(details) => ExpectedResult::success(
            details.is_payable,
            details.zakat_due,
            details.total_assets,
            details.net_assets,
            details.nisab_threshold,
        ),
        Err(e) => ExpectedResult::error(e.code()),
    };

    TestCase {
        id: id.to_string(),
        description: description.to_string(),
        category,
        asset_type: AssetType::Gold,
        config,
        input,
        expected,
    }
}

// ============================================================================
// SILVER HAPPY PATH
// ============================================================================

fn silver_happy_path() -> Vec<TestCase> {
    vec![
        // Silver above nisab
        generate_silver_case(
            "silver_001",
            "Silver 600g above 595g nisab - payable",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "600")
                .field("purity", "1000")
                .field("usage", "investment")
                .hawl(true),
        ),

        // Silver below nisab
        generate_silver_case(
            "silver_002",
            "Silver 500g below 595g nisab - exempt",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "500")
                .field("purity", "1000")
                .field("usage", "investment")
                .hawl(true),
        ),

        // Sterling silver (925) - purity adjustment
        generate_silver_case(
            "silver_003",
            "Sterling silver 925 purity - effective weight reduced",
            TestCategory::HappyPath,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("weight_grams", "650")
                .field("purity", "925")
                .field("usage", "investment")
                .hawl(true),
        ),
    ]
}

fn generate_silver_case(
    id: &str,
    description: &str,
    category: TestCategory,
    config: TestConfig,
    input: TestInput,
) -> TestCase {
    let madhab = match config.madhab.as_str() {
        "shafi" => Madhab::Shafi,
        "maliki" => Madhab::Maliki,
        "hanbali" => Madhab::Hanbali,
        _ => Madhab::Hanafi,
    };

    let gold_price: Decimal = config.gold_price_per_gram.parse().unwrap_or(dec!(0));
    let silver_price: Decimal = config.silver_price_per_gram.parse().unwrap_or(dec!(0));

    let zakat_config = ZakatConfig::new()
        .with_madhab(madhab)
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);

    let weight: Decimal = input.fields.get("weight_grams")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .parse()
        .unwrap_or(dec!(0));
    let purity: u32 = input.fields.get("purity")
        .and_then(|v| v.as_str())
        .unwrap_or("1000")
        .parse()
        .unwrap_or(1000);
    let usage = match input.fields.get("usage").and_then(|v| v.as_str()).unwrap_or("investment") {
        "personal" | "personal_use" => JewelryUsage::PersonalUse,
        _ => JewelryUsage::Investment,
    };
    let liabilities: Decimal = input.liabilities_due_now.parse().unwrap_or(dec!(0));

    let silver = PreciousMetals::silver(weight)
        .purity(purity)
        .usage(usage)
        .add_liability("Liabilities", liabilities)
        .hawl(input.hawl_satisfied);

    let expected = match silver.calculate_zakat(&zakat_config) {
        Ok(details) => ExpectedResult::success(
            details.is_payable,
            details.zakat_due,
            details.total_assets,
            details.net_assets,
            details.nisab_threshold,
        ),
        Err(e) => ExpectedResult::error(e.code()),
    };

    TestCase {
        id: id.to_string(),
        description: description.to_string(),
        category,
        asset_type: AssetType::Silver,
        config,
        input,
        expected,
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

fn edge_cases() -> Vec<TestCase> {
    vec![
        // Zero values - everything zero
        generate_business_case(
            "edge_001",
            "All zero values - exempt",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "0")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Hawl not satisfied
        generate_business_case(
            "edge_002",
            "Hawl not satisfied - exempt regardless of value",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "100000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(false),
        ),

        // Liabilities exceed assets
        generate_business_case(
            "edge_003",
            "Liabilities exceed assets - clamped to zero",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "5000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .liabilities("10000")
                .hawl(true),
        ),

        // Just below nisab (by 1 unit)
        generate_business_case(
            "edge_004",
            "Just below nisab by 1 unit - exempt",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "8499")  // 8500 - 1
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Just above nisab (by 1 unit)
        generate_business_case(
            "edge_005",
            "Just above nisab by 1 unit - payable",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "8501")  // 8500 + 1
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Large values
        generate_business_case(
            "edge_006",
            "Very large value - overflow protection",
            TestCategory::EdgeCase,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "999999999999")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),
    ]
}

// ============================================================================
// PRECISION TESTS
// ============================================================================

fn precision_tests() -> Vec<TestCase> {
    vec![
        // Decimal precision - many decimal places
        generate_business_case(
            "precision_001",
            "High precision decimal input",
            TestCategory::Precision,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "10000.12345678")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Very small decimal that still exceeds nisab
        generate_business_case(
            "precision_002",
            "Decimal just over nisab",
            TestCategory::Precision,
            TestConfig::standard("100", "1"),
            TestInput::new()
                .field("cash_on_hand", "8500.00000001")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),

        // Decimal prices
        generate_gold_case(
            "precision_003",
            "Gold with decimal price",
            TestCategory::Precision,
            TestConfig::standard("99.99", "0.85"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "24")
                .field("usage", "investment")
                .hawl(true),
        ),
    ]
}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

fn configuration_tests() -> Vec<TestCase> {
    vec![
        // Shafi madhab - personal jewelry exempt
        generate_gold_case(
            "config_001",
            "Shafi madhab - personal jewelry exempt",
            TestCategory::Configuration,
            TestConfig::standard("100", "1").with_madhab("shafi"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "24")
                .field("usage", "personal_use")
                .hawl(true),
        ),

        // Hanafi madhab - personal jewelry taxable
        generate_gold_case(
            "config_002",
            "Hanafi madhab - personal jewelry taxable",
            TestCategory::Configuration,
            TestConfig::standard("100", "1").with_madhab("hanafi"),
            TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "24")
                .field("usage", "personal_use")
                .hawl(true),
        ),

        // Different gold prices affect nisab threshold
        generate_business_case(
            "config_003",
            "Higher gold price raises nisab threshold",
            TestCategory::Configuration,
            TestConfig::standard("200", "2"),  // Double prices
            TestInput::new()
                .field("cash_on_hand", "10000")
                .field("inventory_value", "0")
                .field("receivables", "0")
                .hawl(true),
        ),
    ]
}

// ============================================================================
// ERROR TESTS
// ============================================================================

fn error_tests() -> Vec<TestCase> {
    // Note: These tests expect errors. We generate them by trying to trigger
    // validation errors in the Rust library.

    vec![
        // Missing gold price (will cause config error)
        {
            let config = TestConfig {
                gold_price_per_gram: "0".to_string(),
                silver_price_per_gram: "1".to_string(),
                madhab: "hanafi".to_string(),
                nisab_standard: None,
                currency_code: "USD".to_string(),
            };

            let input = TestInput::new()
                .field("cash_on_hand", "10000")
                .hawl(true);

            // Try to execute and capture error
            let zakat_config = ZakatConfig::new()
                .with_madhab(Madhab::Hanafi)
                .with_gold_price(dec!(0))
                .with_silver_price(dec!(1));

            let business = BusinessZakat::new()
                .cash(dec!(10000))
                .hawl(true);

            let expected = match business.calculate_zakat(&zakat_config) {
                Ok(details) => ExpectedResult::success(
                    details.is_payable,
                    details.zakat_due,
                    details.total_assets,
                    details.net_assets,
                    details.nisab_threshold,
                ),
                Err(e) => ExpectedResult::error(e.code()),
            };

            TestCase {
                id: "error_001".to_string(),
                description: "Zero gold price triggers config error".to_string(),
                category: TestCategory::Error,
                asset_type: AssetType::Business,
                config,
                input,
                expected,
            }
        },

        // Negative input (should be clamped or error)
        {
            let config = TestConfig::standard("100", "1");
            let input = TestInput::new()
                .field("cash_on_hand", "-1000")
                .hawl(true);

            let zakat_config = ZakatConfig::new()
                .with_madhab(Madhab::Hanafi)
                .with_gold_price(dec!(100))
                .with_silver_price(dec!(1));

            let business = BusinessZakat::new()
                .cash(dec!(-1000))
                .hawl(true);

            let expected = match business.calculate_zakat(&zakat_config) {
                Ok(details) => ExpectedResult::success(
                    details.is_payable,
                    details.zakat_due,
                    details.total_assets,
                    details.net_assets,
                    details.nisab_threshold,
                ),
                Err(e) => ExpectedResult::error(e.code()),
            };

            TestCase {
                id: "error_002".to_string(),
                description: "Negative input value triggers validation error".to_string(),
                category: TestCategory::Validation,
                asset_type: AssetType::Business,
                config,
                input,
                expected,
            }
        },

        // Invalid purity for gold (> 24)
        {
            let config = TestConfig::standard("100", "1");
            let input = TestInput::new()
                .field("weight_grams", "100")
                .field("purity", "30")  // Invalid: > 24
                .field("usage", "investment")
                .hawl(true);

            let zakat_config = ZakatConfig::new()
                .with_madhab(Madhab::Hanafi)
                .with_gold_price(dec!(100))
                .with_silver_price(dec!(1));

            let gold = PreciousMetals::gold(dec!(100))
                .purity(30)  // Invalid
                .hawl(true);

            let expected = match gold.calculate_zakat(&zakat_config) {
                Ok(details) => ExpectedResult::success(
                    details.is_payable,
                    details.zakat_due,
                    details.total_assets,
                    details.net_assets,
                    details.nisab_threshold,
                ),
                Err(e) => ExpectedResult::error(e.code()),
            };

            TestCase {
                id: "error_003".to_string(),
                description: "Invalid gold purity (>24) triggers validation error".to_string(),
                category: TestCategory::Validation,
                asset_type: AssetType::Gold,
                config,
                input,
                expected,
            }
        },
    ]
}
