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
use zakat_providers::{BestEffortPriceProvider, Prices, PriceProvider, FileSystemPriceCache};

#[cfg(feature = "live-pricing")]
#[cfg(feature = "live-pricing")]
use zakat_providers::BinancePriceProvider;

mod wizard;
mod config_loader;

/// Interactive Zakat Calculator CLI
#[derive(Parser, Debug)]
#[command(name = "zakat")]
#[command(author = "zakatrs contributors")]
#[command(version)]
#[command(about = "Interactive Zakat calculator with live pricing support", long_about = None)]
struct Args {
    /// Enable file logging to logs/ directory
    #[arg(long, default_value = "false")]
    log: bool,

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
    /// Load portfolio from file
    #[arg(long)]
    load: Option<std::path::PathBuf>,

    /// Output results as JSON
    #[arg(long, default_value = "false")]
    json: bool,
}

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
    let args = Args::parse();

    // Initialize tracing with optional file logging
    // The _guard must be held for the duration of main() to flush logs
    let _file_guard: Option<tracing_appender::non_blocking::WorkerGuard>;
    
    if args.log {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        
        let file_appender = tracing_appender::rolling::daily("logs", "zakat.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        _file_guard = Some(guard);
        
        // Console layer for stdout
        let console_layer = tracing_subscriber::fmt::layer();
        
        // File layer with no ANSI colors
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false);
        
        // Use a global env filter for both
        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("zakat=debug".parse().unwrap());
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();
        
        tracing::info!("--- Zakat Calculation Session Started [{}] ---", chrono::Utc::now());
    } else {
        _file_guard = None;
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("zakat=info".parse().unwrap()),
            )
            .init();
    }

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
    let portfolio = if let Some(path) = &args.load {
        println!("{}", format!("📂 Loading portfolio from {:?}...", path).bright_blue());
        load_portfolio(path)?
    } else if args.wizard {
        wizard::run_wizard_mode()?
    } else {
        println!("{}", "💡 Tip: Run with --wizard for a guided step-by-step mode.".dimmed());
        
        let mut p = ZakatPortfolio::new();
        
        loop {
            let choices = vec![
                "➕ Add Asset",
                "✏️ Edit Asset",
                "💾 Save Portfolio",
                "📂 Load Portfolio",
                "🧮 Calculate & Exit",
            ];

            let choice = Select::new("Menu:", choices).prompt()?;

            match choice {
                "➕ Add Asset" => {
                    match add_asset_interactive() {
                        Ok(Some(item)) => {
                            p = p.add(item);
                            println!("{}", "✓ Asset added successfully!".green());
                        }
                        Ok(None) => println!("{}", "⚠ Skipped asset entry.".yellow()),
                        Err(e) => println!("{} {}", "✗ Error:".red(), e),
                    }
                }
                "✏️ Edit Asset" => {
                    match edit_asset_interactive(&mut p) {
                        Ok(true) => println!("{}", "✓ Asset updated successfully!".green()),
                        Ok(false) => println!("{}", "⚠ Edit cancelled.".yellow()),
                        Err(e) => println!("{} {}", "✗ Error:".red(), e),
                    }
                }
                "💾 Save Portfolio" => {
                    let filename = Text::new("Filename to save (e.g. my_zakat.json):")
                        .with_default("portfolio.json")
                        .prompt()?;
                    save_portfolio(&p, &filename)?;
                }
                "📂 Load Portfolio" => {
                    let filename = Text::new("Filename to load:")
                        .with_default("portfolio.json")
                        .prompt()?;
                    match load_portfolio(std::path::Path::new(&filename)) {
                        Ok(loaded) => {
                            p = loaded;
                            println!("{}", "✓ Portfolio loaded successfully!".green());
                        }
                        Err(e) => println!("{} {}", "✗ Load Error:".red(), e),
                    }
                }
                "🧮 Calculate & Exit" => break,
                _ => {}
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
    display_results(&result, &config, args.json);

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
    use indicatif::{ProgressBar, ProgressStyle};
    
    // Default fallback prices
    let fallback = Prices::new(
        args.gold_price.unwrap_or(dec!(85)),
        args.silver_price.unwrap_or(dec!(1)),
    )?;

    if args.offline {
        println!("{}", "📴 Running in offline mode with static prices.".yellow());
        return Ok(fallback);
    }

    // Create a spinner for visual feedback
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message("Fetching live prices...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    #[cfg(feature = "live-pricing")]
    {
        let binance = BinancePriceProvider::default();
        
        #[cfg(not(target_arch = "wasm32"))]
        let primary_provider = FileSystemPriceCache::new(binance, std::time::Duration::from_secs(3600)); 
        
        #[cfg(target_arch = "wasm32")]
        let primary_provider = binance;

        let provider = BestEffortPriceProvider::new(
            primary_provider,
            fallback.clone(),
        );
        
        match provider.get_prices().await {
            Ok(prices) => {
                if prices.gold_per_gram > Decimal::ZERO {
                    spinner.finish_with_message("✓ Live prices fetched successfully!");
                    // If silver price is zero (Binance doesn't support silver), use fallback
                    let final_silver = if prices.silver_per_gram.is_zero() {
                        warn!("Silver price is zero from provider, using fallback silver price");
                        fallback.silver_per_gram
                    } else {
                        prices.silver_per_gram
                    };
                    return Ok(Prices::new(prices.gold_per_gram, final_silver)?);
                }
            }
            Err(e) => {
                warn!("Live pricing failed: {}", e);
            }
        }
    }

    spinner.finish_with_message("⚠ Using fallback prices.");
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

fn display_results(result: &PortfolioResult, config: &ZakatConfig, json_mode: bool) {
    if json_mode {
        let json_output = serde_json::to_string_pretty(result).unwrap_or_else(|e| format!("{{\"error\": \"Serialization failed: {}\"}}", e));
        println!("{}", json_output);
        return;
    }
    use tabled::settings::{Width, Modify, object::Columns, Alignment};
    
    let all_results = result.results(); // Use reconstruction method
    let rows: Vec<ResultRow> = all_results.iter().filter_map(|r| {
        match r {
            PortfolioItemResult::Success { details, .. } => {
                // With tabled "ansi" feature, colors work correctly
                let status = if details.is_payable {
                    "✓ PAYABLE".green().to_string()
                } else {
                    "EXEMPT".yellow().to_string()
                };

                let rec = match details.recommendation {
                    ZakatRecommendation::Obligatory => "Pay 2.5%".green().to_string(),
                    ZakatRecommendation::Recommended => "Sadaqah".bright_cyan().to_string(),
                    ZakatRecommendation::None => "-".dimmed().to_string(),
                };

                Some(ResultRow {
                    label: details.label.clone().unwrap_or_else(|| "Asset".to_string()),
                    wealth_type: format!("{:?}", details.wealth_type),
                    net_assets: config.format_currency(details.net_assets),
                    nisab: config.format_currency(details.nisab_threshold),
                    status,
                    zakat_due: if details.is_payable {
                        config.format_currency(details.zakat_due).green().to_string()
                    } else {
                        config.format_currency(details.zakat_due)
                    },
                    recommendation: rec,
                })
            }
            PortfolioItemResult::Failure { source, error, .. } => {
                // Extract a clean, short error message
                let short_error = match error {
                    zakat_core::types::ZakatError::ConfigurationError(details) => {
                        details.suggestion.clone().unwrap_or_else(|| details.reason_key.clone())
                    },
                    zakat_core::types::ZakatError::InvalidInput(details) => {
                        format!("{}: {}", details.field, details.reason_key)
                    },
                    _ => error.report(),
                };
                // Truncate if too long
                let display_error = if short_error.len() > 35 {
                    format!("{}...", &short_error[..32])
                } else {
                    short_error
                };

                Some(ResultRow {
                    label: source.clone(),
                    wealth_type: "ERROR".red().to_string(),
                    net_assets: "-".to_string(),
                    nisab: "-".to_string(),
                    status: "✗ FAILED".red().to_string(),
                    zakat_due: "-".to_string(),
                    recommendation: display_error.red().to_string(),
                })
            }
        }
    }).collect();

    let table = Table::new(rows)
        .with(Style::rounded())
        .with(Modify::new(Columns::single(2)).with(Alignment::right()))
        .with(Modify::new(Columns::single(3)).with(Alignment::right()))
        .with(Modify::new(Columns::single(5)).with(Alignment::right()))
        .with(Modify::new(Columns::single(6)).with(Width::truncate(40)))
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

fn save_portfolio(portfolio: &ZakatPortfolio, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(portfolio)?;
    std::fs::write(filename, json)?;
    println!("{} {}", "✓ Portfolio saved to:".green(), filename);
    Ok(())
}

fn load_portfolio(path: &std::path::Path) -> Result<ZakatPortfolio, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let portfolio: ZakatPortfolio = serde_json::from_str(&content)?;
    Ok(portfolio)
}

// =============================================================================
// Edit Asset Functions
// =============================================================================

/// Interactive prompt to edit an existing asset in the portfolio.
/// Returns Ok(true) if asset was updated, Ok(false) if cancelled.
fn edit_asset_interactive(portfolio: &mut ZakatPortfolio) -> Result<bool, Box<dyn std::error::Error>> {
    use zakat_core::traits::CalculateZakat;
    
    let items = portfolio.get_items();
    
    // Check if portfolio is empty
    if items.is_empty() {
        println!("{}", "⚠ No assets in portfolio to edit.".yellow());
        return Ok(false);
    }
    
    // Build selection list with format: [index] label (type)
    let choices: Vec<String> = items.iter().enumerate().map(|(idx, item)| {
        let label = CalculateZakat::get_label(item).unwrap_or_else(|| format!("Asset #{}", idx + 1));
        let type_name = match item {
            PortfolioItem::Business(_) => "Business",
            PortfolioItem::PreciousMetals(_) => "Precious Metal",
            PortfolioItem::Investment(_) => "Investment",
            PortfolioItem::Agriculture(_) => "Agriculture",
            PortfolioItem::Income(_) => "Income",
            PortfolioItem::Livestock(_) => "Livestock",
            PortfolioItem::Mining(_) => "Mining",
            PortfolioItem::Fitrah(_) => "Fitrah",
            PortfolioItem::Custom(_) => "Custom",
        };
        format!("[{}] {} ({})", idx + 1, label, type_name)
    }).collect();
    
    let mut cancel_choices = choices.clone();
    cancel_choices.push("❌ Cancel".to_string());
    
    let selection = Select::new("Select asset to edit:", cancel_choices).prompt()?;
    
    if selection == "❌ Cancel" {
        return Ok(false);
    }
    
    // Find the selected index
    let selected_idx = choices.iter().position(|c| c == &selection).ok_or("Invalid selection")?;
    let selected_item = &items[selected_idx];
    let asset_id = CalculateZakat::get_id(selected_item);
    
    // Edit based on type
    let new_item: Option<PortfolioItem> = match selected_item {
        PortfolioItem::Business(asset) => edit_business_asset(asset)?,
        PortfolioItem::PreciousMetals(asset) => edit_precious_metal_asset(asset)?,
        PortfolioItem::Investment(asset) => edit_investment_asset(asset)?,
        PortfolioItem::Agriculture(asset) => edit_agriculture_asset(asset)?,
        PortfolioItem::Income(_) => {
            println!("{}", "⚠ Income asset editing not yet supported.".yellow());
            None
        }
        PortfolioItem::Livestock(_) => {
            println!("{}", "⚠ Livestock asset editing not yet supported.".yellow());
            None
        }
        PortfolioItem::Mining(_) => {
            println!("{}", "⚠ Mining asset editing not yet supported.".yellow());
            None
        }
        PortfolioItem::Fitrah(_) => {
            println!("{}", "⚠ Fitrah asset editing not yet supported.".yellow());
            None
        }
        PortfolioItem::Custom(_) => {
            println!("{}", "⚠ Custom asset editing not yet supported.".yellow());
            None
        }
    };
    
    if let Some(item) = new_item {
        portfolio.replace(asset_id, item)?;
        return Ok(true);
    }
    
    Ok(false)
}

fn edit_business_asset(asset: &BusinessZakat) -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    println!("\n{}", "--- Editing Business Asset ---".bright_blue());
    
    let label = Text::new("Asset label:")
        .with_default(&asset.label.clone().unwrap_or_else(|| "Business".to_string()))
        .prompt()?;
    
    let cash: Decimal = CustomType::new("Cash on hand ($):")
        .with_default(asset.cash_on_hand)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let inventory: Decimal = CustomType::new("Inventory value ($):")
        .with_default(asset.inventory_value)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let receivables: Decimal = CustomType::new("Accounts receivable ($):")
        .with_default(asset.receivables)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let liabilities: Decimal = CustomType::new("Liabilities/debts due now ($):")
        .with_default(asset.total_liabilities())
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let new_asset = BusinessZakat::new()
        .label(label)
        .cash(cash)
        .inventory(inventory)
        .receivables(receivables)
        .add_liability("Liabilities", liabilities);
    
    Ok(Some(new_asset.into()))
}

fn edit_precious_metal_asset(asset: &PreciousMetals) -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    println!("\n{}", "--- Editing Precious Metal Asset ---".bright_yellow());
    
    let is_gold = matches!(asset.metal_type, Some(WealthType::Gold));
    let metal_name = if is_gold { "Gold" } else { "Silver" };
    
    let label = Text::new(&format!("{} asset label:", metal_name))
        .with_default(&asset.label.clone().unwrap_or_else(|| metal_name.to_string()))
        .prompt()?;
    
    let weight: Decimal = CustomType::new(&format!("{} weight in grams:", metal_name))
        .with_default(asset.weight_grams)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let new_asset = if is_gold {
        PreciousMetals::gold(weight).label(label)
    } else {
        PreciousMetals::silver(weight).label(label)
    };
    
    Ok(Some(new_asset.into()))
}

fn edit_investment_asset(asset: &InvestmentAssets) -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    println!("\n{}", "--- Editing Investment Asset ---".bright_magenta());
    
    let label = Text::new("Investment label:")
        .with_default(&asset.label.clone().unwrap_or_else(|| "Investments".to_string()))
        .prompt()?;
    
    let market_value: Decimal = CustomType::new("Current market value ($):")
        .with_default(asset.value)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let new_asset = InvestmentAssets::new()
        .label(label)
        .value(market_value)
        .kind(asset.investment_type);
    
    Ok(Some(new_asset.into()))
}

fn edit_agriculture_asset(asset: &AgricultureAssets) -> Result<Option<PortfolioItem>, Box<dyn std::error::Error>> {
    println!("\n{}", "--- Editing Agriculture Asset ---".bright_green());
    
    let label = Text::new("Crop/Harvest label:")
        .with_default(&asset.label.clone().unwrap_or_else(|| "Harvest".to_string()))
        .prompt()?;
    
    let weight: Decimal = CustomType::new("Harvest weight in kg:")
        .with_default(asset.harvest_weight_kg)
        .with_error_message("Please enter a valid number")
        .prompt()?;
    
    let is_irrigated = matches!(asset.irrigation, IrrigationMethod::Irrigated);
    let irrigated = Confirm::new("Was it irrigated artificially (vs rain-fed)?")
        .with_default(is_irrigated)
        .prompt()?;
    
    let method = if irrigated {
        IrrigationMethod::Irrigated
    } else {
        IrrigationMethod::Rain
    };
    
    let new_asset = AgricultureAssets::new()
        .label(label)
        .harvest_weight(weight)
        .irrigation(method);
    
    Ok(Some(new_asset.into()))
}

