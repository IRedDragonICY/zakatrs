//! # Zakat Test Generator
//!
//! This binary generates the golden data JSON file (`zakat_suite.json`)
//! containing test inputs and Rust-calculated expected outputs.
//!
//! All polyglot bindings (Python, Dart, TypeScript) run against this JSON
//! to ensure they match the Rust core exactly.
//!
//! ## Usage
//! ```sh
//! cargo run -p zakat-test-gen
//! ```
//!
//! This will generate `tests/fixtures/zakat_suite.json`.

mod schema;
mod scenarios;

use schema::{ComplianceSuite, SuiteMeta};
use std::fs;
use std::path::Path;

fn main() {
    println!("🧪 Zakat Compliance Test Suite Generator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Generate all test cases
    println!("📊 Generating test scenarios...");
    let cases = scenarios::generate_all_scenarios();
    println!("   Generated {} test cases\n", cases.len());

    // Print summary by category
    let mut happy_path = 0;
    let mut edge_cases = 0;
    let mut precision = 0;
    let mut config = 0;
    let mut errors = 0;
    let mut validation = 0;

    for case in &cases {
        match case.category {
            schema::TestCategory::HappyPath => happy_path += 1,
            schema::TestCategory::EdgeCase => edge_cases += 1,
            schema::TestCategory::Precision => precision += 1,
            schema::TestCategory::Configuration => config += 1,
            schema::TestCategory::Error => errors += 1,
            schema::TestCategory::Validation => validation += 1,
        }
    }

    println!("📋 Test Case Summary:");
    println!("   • Happy Path:    {}", happy_path);
    println!("   • Edge Cases:    {}", edge_cases);
    println!("   • Precision:     {}", precision);
    println!("   • Configuration: {}", config);
    println!("   • Error Cases:   {}", errors);
    println!("   • Validation:    {}", validation);
    println!();

    // Build the suite
    let suite = ComplianceSuite {
        meta: SuiteMeta {
            schema_version: "1.0.0".to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
            total_cases: cases.len(),
        },
        cases,
    };

    // Ensure the output directory exists
    let output_dir = Path::new("tests/fixtures");
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).expect("Failed to create tests/fixtures directory");
    }

    // Write to JSON file
    let output_path = output_dir.join("zakat_suite.json");
    let json = serde_json::to_string_pretty(&suite).expect("Failed to serialize suite");
    fs::write(&output_path, &json).expect("Failed to write zakat_suite.json");

    println!("✅ Generated: {}", output_path.display());
    println!("   File size: {} bytes", json.len());
    println!("\n🎉 Done! Run polyglot tests with:");
    println!("   • Python: pytest tests/py/test_compliance.py");
    println!("   • Dart:   cd zakat_dart && flutter test");
    println!("   • TS:     npm test (in pkg/)");
}
