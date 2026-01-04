//! # Zakat CLI - Interactive Zakat Calculator
//!
//! An interactive command-line tool for calculating Zakat on various assets.
//!
//! ## Features
//! - Interactive prompts for asset input
//! - Live price fetching with fallback support
//! - Pretty-printed results with calculation breakdown
//! - Supports multiple asset types (Business, Gold, Silver, etc.)
//!
//! ## Usage
//! ```bash
//! # Run the interactive calculator
//! zakat
//!
//! # With verbose logging
//! RUST_LOG=info zakat
//! ```

use clap::Parser;
use colored::Colorize;
use inquire::{Confirm, CustomType, Select, Text};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use tabled::{Table, Tabled, settings::Style};
use tracing::warn;

use zakat_core::assets::PortfolioItem;
use zakat_core::prelude::*;
use zakat_providers::{BestEffortPriceProvider, Prices, PriceProvider};

#[cfg(feature = "live-pricing")]
#[cfg(feature = "live-pricing")]
use zakat_providers::BinancePriceProvider;

mod wizard;

/// Interactive Zakat Calculator CLI
#[derive(Parser, Debug)]
#[command(name = "zakat")]
#[command(author = "zakatrs contributors")]
#[command(version)]
#[command(about = "Interactive Zakat calculator with live pricing support", long_about = None)]
struct Args {
    /// Use static prices instead of fetching live
    #[arg(long, default_value = "false")]
    offline: bool,

    /// Gold price per gram (overrides live/default)
    #[arg(long)]
    gold_price: Option<Decimal>,

    /// Silver price per gram (overrides live/default)
    #[arg(long)]
    silver_price: Option<Decimal>,

    /// Enable verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    /// Run the interactive guided wizard
    #[arg(long, default_value = "false")]
    wizard: bool,
}

/// Displayable row for the results table
#[derive(Tabled)]
struct ResultRow {
    #[tabled(rename = "Asset")]
    label: String,
    #[tabled(rename = "Type")]
    wealth_type: String,
    #[tabled(rename = "Net Assets")]
    net_assets: String,
    #[tabled(rename = "Nisab")]
    nisab: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Zakat Due")]
    zakat_due: String,
    #[tabled(rename = "Recommendation")]
    recommendation: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("zakat=info".parse().unwrap()),
        )
        .init();

    let args = Args::parse();

    println!("\n{}", "═══════════════════════════════════════════════".bright_cyan());
    println!("{}", "        🕌  ZAKAT CALCULATOR CLI  🕌           ".bright_cyan().bold());
    println!("{}", "═══════════════════════════════════════════════".bright_cyan());
    println!();

    // Step 1: Get prices
    let prices = get_prices(&args).await?;
    
    println!(
        "📊 {} Gold: {}/g | Silver: {}/g",
        "Using prices:".bright_green(),
        format!("${:.2}", prices.gold_per_gram).yellow(),
        format!("${:.2}", prices.silver_per_gram).yellow()
    );
    println!();
    println!();

    // Step 2: Build config
    let config = ZakatConfig::new()
        .with_gold_price(prices.gold_per_gram)
        .with_silver_price(prices.silver_per_gram)
        .with_madhab(Madhab::Hanafi)
        .with_nisab_standard(NisabStandard::Gold)
        .with_currency_code("USD");

    // Step 3: Interactive asset loop (Standard or Wizard)
    let portfolio = if args.wizard {
        wizard::run_wizard_mode()?
    } else {
        println!("{}", "💡 Tip: Run with --wizard for a guided step-by-step mode.".dimmed());
        
        let mut p = ZakatPortfolio::new();
        
        loop {
            let add_more = if p.get_items().is_empty() {
                true
            } else {
                Confirm::new("Add another asset?")
                    .with_default(true)
                    .prompt()?
            };

            if !add_more {
                break;
            }

            match add_asset_interactive() {
                Ok(Some(item)) => {
                    p = p.add(item);
                    println!("{}", "✓ Asset added successfully!".green());
                }
                Ok(None) => {
                    println!("{}", "⚠ Skipped asset entry.".yellow());
                }
                Err(e) => {
                    println!("{} {}", "✗ Error:".red(), e);
                }
            }
            println!();
        }
        p
    };

    if portfolio.get_items().is_empty() {
        println!("{}", "No assets added. Exiting.".yellow());
        return Ok(());
    }

    // Step 4: Calculate
    println!();
    println!("{}", "Calculating Zakat...".bright_blue());
    println!();

    let result = portfolio.calculate_total(&config);

    // Step 5: Display results
    display_results(&result, &config);

    // Step 6: Offer to save snapshot
    if Confirm::new("Save calculation snapshot for audit?")
        .with_default(false)
        .prompt()?
    {
        let snapshot = portfolio.snapshot(&config, &result)
            .with_metadata("calculated_at", chrono::Utc::now().to_rfc3339())
            .with_metadata("cli_version", env!("CARGO_PKG_VERSION"));

        let filename = format!("zakat_snapshot_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        std::fs::write(&filename, snapshot.to_json()?)?;
        println!("{} {}", "✓ Snapshot saved to:".green(), filename.bright_white());
    }

    println!();
    println!("{}", "May Allah accept your Zakat! 🤲".bright_green().bold());
    println!();

    Ok(())
}

/// Fetches prices using BestEffortPriceProvider
async fn get_prices(args: &Args) -> Result<Prices, Box<dyn std::error::Error>> {
    // Default fallback prices
    let fallback = Prices::new(
        args.gold_price.unwrap_or(dec!(85)),
        args.silver_price.unwrap_or(dec!(1)),
    )?;

    if args.offline {
        println!("{}", "📴 Running in offline mode with static prices.".yellow());
        return Ok(fallback);
    }

    // Try to fetch live prices
    println!("{}", "🌐 Fetching live prices...".bright_blue());

    #[cfg(feature = "live-pricing")]
    {
        let provider = BestEffortPriceProvider::new(
            BinancePriceProvider::default(),
            fallback.clone(),
        );
        
        match provider.get_prices().await {
            Ok(prices) => {
                if prices.gold_per_gram > Decimal::ZERO {
                    println!("{}", "✓ Live prices fetched successfully!".green());
                    return Ok(prices);
                }
            }
            Err(e) => {
                warn!("Live pricing failed: {}", e);
            }
        }
    }

    println!("{}", "⚠ Using fallback prices.".yellow());
    Ok(fallback)
}

/// Interactive prompt to add an asset
fn add_asset_interactive() -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let asset_types = vec![
        "💼 Business Assets",
        "🥇 Gold",
        "🥈 Silver",
        "💰 Cash/Savings",
        "📈 Investments",
        "🌾 Agriculture",
        "❌ Cancel",
    ];

    let selection = Select::new("Select asset type:", asset_types).prompt()?;

    match selection {
        "💼 Business Assets" => add_business_asset(),
        "🥇 Gold" => add_precious_metal_asset(true),
        "🥈 Silver" => add_precious_metal_asset(false),
        "💰 Cash/Savings" => add_cash_asset(),
        "📈 Investments" => add_investment_asset(),
        "🌾 Agriculture" => add_agriculture_asset(),
        "❌ Cancel" => Ok(None),
        _ => Ok(None),
    }
}

fn add_business_asset() -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let label = Text::new("Asset label (e.g., 'Main Store'):")
        .with_default("Business")
        .prompt()?;

    let cash: Decimal = CustomType::new("Cash on hand ($):")
        .with_default(dec!(0))
        .prompt()?;

    let inventory: Decimal = CustomType::new("Inventory value ($):")
        .with_default(dec!(0))
        .prompt()?;

    let receivables: Decimal = CustomType::new("Accounts receivable ($):")
        .with_default(dec!(0))
        .prompt()?;

    let liabilities: Decimal = CustomType::new("Liabilities/debts due now ($):")
        .with_default(dec!(0))
        .prompt()?;

    let asset = BusinessZakat::new()
        .label(label)
        .cash(cash)
        .inventory(inventory)
        .receivables(receivables)
        .add_liability("Liabilities", liabilities);

    Ok(Some(asset.into()))
}

fn add_precious_metal_asset(is_gold: bool) -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let metal_name = if is_gold { "Gold" } else { "Silver" };
    
    let label = Text::new(&format!("{} asset label:", metal_name))
        .with_default(metal_name)
        .prompt()?;

    let weight: Decimal = CustomType::new(&format!("{} weight in grams:", metal_name))
        .with_default(dec!(0))
        .prompt()?;

    let asset = if is_gold {
        PreciousMetals::gold(weight).label(label)
    } else {
        PreciousMetals::silver(weight).label(label)
    };

    Ok(Some(asset.into()))
}

fn add_cash_asset() -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let label = Text::new("Cash/Savings label:")
        .with_default("Savings Account")
        .prompt()?;

    let amount: Decimal = CustomType::new("Total cash/savings ($):")
        .with_default(dec!(0))
        .prompt()?;

    let asset = BusinessZakat::new()
        .label(label)
        .cash(amount);

    Ok(Some(asset.into()))
}

fn add_investment_asset() -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let label = Text::new("Investment label:")
        .with_default("Stocks")
        .prompt()?;

    let market_value: Decimal = CustomType::new("Current market value ($):")
        .with_default(dec!(0))
        .prompt()?;

    let asset = InvestmentAssets::new()
        .label(label)
        .value(market_value)
        .kind(InvestmentType::Stock);

    Ok(Some(asset.into()))
}

fn add_agriculture_asset() -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    let label = Text::new("Crop/Harvest label:")
        .with_default("Rice Harvest")
        .prompt()?;

    let weight: Decimal = CustomType::new("Harvest weight in kg:")
        .with_default(dec!(0))
        .prompt()?;

    let irrigated = Confirm::new("Was it irrigated artificially (vs rain-fed)?")
        .with_default(false)
        .prompt()?;

    let method = if irrigated {
        IrrigationMethod::Irrigated
    } else {
        IrrigationMethod::Rain
    };

    let asset = AgricultureAssets::new()
        .label(label)
        .harvest_weight(weight)
        .irrigation(method);

    Ok(Some(asset.into()))
}

fn display_results(result: &PortfolioResult, config: &ZakatConfig) {
    let rows: Vec<ResultRow> = result.results.iter().filter_map(|r| {
        match r {
            PortfolioItemResult::Success { details, .. } => {
                let status = if details.is_payable {
                    "✓ PAYABLE".green().to_string()
                } else {
                    "EXEMPT".yellow().to_string()
                };

                let rec = match details.recommendation {
                    ZakatRecommendation::Obligatory => "Obligatory".to_string(),
                    ZakatRecommendation::Recommended => "Sadaqah Recommended".bright_cyan().to_string(),
                    ZakatRecommendation::None => "-".to_string(),
                };

                Some(ResultRow {
                    label: details.label.clone().unwrap_or_else(|| "Asset".to_string()),
                    wealth_type: format!("{:?}", details.wealth_type),
                    net_assets: config.format_currency(details.net_assets),
                    nisab: config.format_currency(details.nisab_threshold),
                    status,
                    zakat_due: config.format_currency(details.zakat_due),
                    recommendation: rec,
                })
            }
            PortfolioItemResult::Failure { source, error, .. } => {
                Some(ResultRow {
                    label: source.clone(),
                    wealth_type: "ERROR".red().to_string(),
                    net_assets: "-".to_string(),
                    nisab: "-".to_string(),
                    status: "✗ FAILED".red().to_string(),
                    zakat_due: "-".to_string(),
                    recommendation: error.to_string(),
                })
            }
        }
    }).collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .to_string();

    println!("{}", table);
    println!();

    // Summary
    println!("{}", "═══════════════════════════════════════════════".bright_cyan());
    println!(
        "  {} {}",
        "TOTAL ZAKAT DUE:".bright_white().bold(),
        config.format_currency(result.total_zakat_due).green().bold()
    );
    println!(
        "  {} {}",
        "Total Assets:".bright_white(),
        config.format_currency(result.total_assets)
    );
    println!(
        "  {} {:?}",
        "Status:".bright_white(),
        result.status
    );
    println!("{}", "═══════════════════════════════════════════════".bright_cyan());
}
