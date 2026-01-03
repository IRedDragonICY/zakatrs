//! Generates JSON Schema files for frontend validation.
//!
//! This example dumps the JSON Schema for each major asset type and
//! core types to the `schemas/` directory.
//!
//! Run with: `cargo run --example dump_schema`

use std::fs;

fn main() {
    // Create schemas directory
    fs::create_dir_all("schemas").expect("Failed to create schemas directory");

    println!("Generating JSON Schemas...");

    // Generate schema for PortfolioItem (the root enum)
    let portfolio_schema = schemars::schema_for!(zakat::assets::PortfolioItem);
    write_schema("portfolio_item", &portfolio_schema);

    // Generate schema for ZakatDetails (output type)
    let details_schema = schemars::schema_for!(zakat::types::ZakatDetails);
    write_schema("zakat_details", &details_schema);

    // Generate schema for ZakatExplanation (structured output)
    let explanation_schema = schemars::schema_for!(zakat::types::ZakatExplanation);
    write_schema("zakat_explanation", &explanation_schema);

    // Generate schema for individual asset types
    let business_schema = schemars::schema_for!(zakat::maal::business::BusinessZakat);
    write_schema("business_zakat", &business_schema);

    let precious_metals_schema = schemars::schema_for!(zakat::maal::precious_metals::PreciousMetals);
    write_schema("precious_metals", &precious_metals_schema);

    let income_schema = schemars::schema_for!(zakat::maal::income::IncomeZakatCalculator);
    write_schema("income_zakat", &income_schema);

    let investment_schema = schemars::schema_for!(zakat::maal::investments::InvestmentAssets);
    write_schema("investment_assets", &investment_schema);

    // Generate schema for supporting enums
    let wealth_type_schema = schemars::schema_for!(zakat::types::WealthType);
    write_schema("wealth_type", &wealth_type_schema);

    let payment_payload_schema = schemars::schema_for!(zakat::types::PaymentPayload);
    write_schema("payment_payload", &payment_payload_schema);

    println!("\n✓ All schemas generated successfully!");
    println!("  Output directory: schemas/");
}

fn write_schema(name: &str, schema: &schemars::schema::RootSchema) {
    let json = serde_json::to_string_pretty(schema).expect("Failed to serialize schema");
    let path = format!("schemas/{}.json", name);
    fs::write(&path, json).expect("Failed to write schema file");
    println!("  ✓ {}", path);
}
