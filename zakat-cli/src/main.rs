//! # Zakat CLI - Interactive Zakat Calculator TUI
//!
//! A modern terminal user interface for calculating Zakat on various assets.
//!
//! ## Features
//! - Interactive TUI with keyboard navigation
//! - Live price fetching with fallback support
//! - Pretty-printed results with calculation breakdown
//! - Supports multiple asset types (Business, Gold, Silver, etc.)
//!
//! ## Usage
//! ```bash
//! # Run the interactive TUI
//! zakat-cli
//!
//! # With offline mode (skip price fetching)
//! zakat-cli --offline
//!
//! # Load existing portfolio
//! zakat-cli --load portfolio.json
//!
//! # Run system diagnostics
//! zakat-cli doctor
//! ```

use clap::{Parser, Subcommand};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::env;
use std::io;
use tracing::warn;

use zakat_providers::{BestEffortPriceProvider, FileSystemPriceCache, PriceProvider, Prices};

#[cfg(feature = "live-pricing")]
use zakat_providers::BinancePriceProvider;

mod config_loader;
mod tui;

use tui::{handle_events, ui, App};

/// Interactive Zakat Calculator CLI
#[derive(Parser, Debug)]
#[command(name = "zakat-cli")]
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

    /// Load portfolio from file
    #[arg(long)]
    load: Option<std::path::PathBuf>,

    /// Output results as JSON (non-interactive mode)
    #[arg(long, default_value = "false")]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run diagnostics to check system health and connectivity
    Doctor,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize tracing with optional file logging
    // NOTE: In TUI mode, we only log to file (no console) to avoid corrupting the UI
    let _file_guard: Option<tracing_appender::non_blocking::WorkerGuard>;
    let is_tui_mode = args.command.is_none(); // TUI mode if no subcommand

    if args.log {
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;

        std::fs::create_dir_all("logs")?;

        let file_appender = tracing_appender::rolling::daily("logs", "zakat.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        _file_guard = Some(guard);

        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("zakat=debug".parse().unwrap());

        // Only add console layer if NOT in TUI mode
        if is_tui_mode {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(file_layer)
                .init();
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer())
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(non_blocking)
                        .with_ansi(false),
                )
                .init();
        }

        tracing::info!(
            "--- Zakat Calculation Session Started [{}] ---",
            chrono::Utc::now()
        );
    } else {
        _file_guard = None;
        // In TUI mode without --log, completely disable tracing to stdout
        if !is_tui_mode {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::from_default_env()
                        .add_directive("zakat=info".parse().unwrap()),
                )
                .init();
        }
        // For TUI mode without --log, we don't initialize any tracing
    }

    // Handle Subcommands (run outside TUI)
    if let Some(Commands::Doctor) = args.command {
        return run_doctor().await;
    }

    // Run TUI
    run_tui(args).await
}

/// Run the TUI application
async fn run_tui(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Create app state
    let mut app = App::new(args.offline);

    // Load portfolio if specified
    if let Some(path) = &args.load
        && let Err(e) = app.load_portfolio(path.to_string_lossy().as_ref()) {
            eprintln!("Warning: Could not load portfolio: {}", e);
        }

    // Fetch prices before starting TUI (async operation)
    let prices = get_prices(&args).await;
    app.set_prices(prices);

    // Initialize terminal
    let mut terminal = ratatui::init();

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    ratatui::restore();

    result
}

/// Main application loop
fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Draw the UI
        terminal.draw(|frame| ui(frame, app))?;

        // Handle events
        if handle_events(app)? {
            break;
        }
    }

    Ok(())
}

/// Fetches prices using BestEffortPriceProvider
async fn get_prices(args: &Args) -> Prices {
    // Default fallback prices
    let fallback = Prices::new(
        args.gold_price.unwrap_or(dec!(85)),
        args.silver_price.unwrap_or(dec!(1)),
    )
    .unwrap();

    if args.offline {
        return fallback;
    }

    #[cfg(feature = "live-pricing")]
    {
        let binance = BinancePriceProvider::default();

        #[cfg(not(target_arch = "wasm32"))]
        let primary_provider =
            FileSystemPriceCache::new(binance, std::time::Duration::from_secs(3600));

        #[cfg(target_arch = "wasm32")]
        let primary_provider = binance;

        let provider = BestEffortPriceProvider::new(primary_provider, fallback.clone());

        match provider.get_prices().await {
            Ok(prices) => {
                if prices.gold_per_gram > Decimal::ZERO {
                    // If silver price is zero, use fallback
                    let final_silver = if prices.silver_per_gram.is_zero() {
                        warn!("Silver price is zero from provider, using fallback");
                        fallback.silver_per_gram
                    } else {
                        prices.silver_per_gram
                    };
                    return Prices::new(prices.gold_per_gram, final_silver).unwrap();
                }
            }
            Err(e) => {
                warn!("Live pricing failed: {}", e);
            }
        }
    }

    fallback
}

/// Run doctor diagnostics (outside TUI)
async fn run_doctor() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚑 Zakat CLI Doctor - Diagnostics Tool");
    println!("═══════════════════════════════════════════════\n");

    // 1. Environment Info
    println!("1. System Information:");
    println!("   OS: {}", std::env::consts::OS);
    println!("   Arch: {}", std::env::consts::ARCH);
    println!("   CLI Version: {}", env!("CARGO_PKG_VERSION"));
    println!(
        "   NO_COLOR: {}",
        if env::var("NO_COLOR").is_ok() {
            "Set (True)"
        } else {
            "Unset"
        }
    );

    // 2. Network Connectivity
    println!("\n2. Network & Pricing:");

    #[cfg(feature = "live-pricing")]
    {
        println!("   Live Pricing Feature: Enabled");
        print!("   Connecting to Binance API... ");
        use std::io::Write;
        io::stdout().flush()?;

        let provider = BinancePriceProvider::default();
        match provider.get_prices().await {
            Ok(prices) => {
                if prices.gold_per_gram > Decimal::ZERO {
                    println!("✓ OK");
                    println!("   Gold: ${:.2}/g", prices.gold_per_gram);
                    println!("   Silver: ${:.2}/g", prices.silver_per_gram);
                } else {
                    println!("⚠ Connected but returned zero prices");
                }
            }
            Err(e) => {
                println!("✗ FAILED");
                println!("   Error: {}", e);
            }
        }
    }

    #[cfg(not(feature = "live-pricing"))]
    {
        println!("   Live Pricing Feature: Disabled (Compiled without 'live-pricing')");
    }

    // 3. Storage
    println!("\n3. Storage:");
    let current_dir = std::env::current_dir()?;
    println!("   Current Directory: {:?}", current_dir);
    println!(
        "   Write Access: {}",
        if !std::fs::metadata(&current_dir)?.permissions().readonly() {
            "Yes"
        } else {
            "No"
        }
    );

    println!("\nDiagnostics Complete.\n");
    Ok(())
}
