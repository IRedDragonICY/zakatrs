use flutter_rust_bridge::frb;
use zakat::prelude::*;
use rust_decimal::prelude::*;
use std::str::FromStr;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

// --- Data Types ---

pub struct DartZakatResult {
    pub zakat_due: f64,
    pub is_payable: bool,
    pub nisab_threshold: f64,
    pub wealth_amount: f64,
    pub limit_name: String,
}

// --- API Methods ---

#[frb(sync)]
pub fn calculate_business_zakat(
    cash: String,
    inventory: String,
    receivables: String,
    liabilities: String,
    gold_price: String,
    silver_price: String,
) -> Result<DartZakatResult, String> {
    // Helper to parse decimal or return error
    let parse = |s: &str, name: &str| -> Result<Decimal, String> {
        Decimal::from_str(s).map_err(|e| format!("Invalid {}: {}", name, e))
    };

    let gold_price_dec = parse(&gold_price, "gold_price")?;
    let silver_price_dec = parse(&silver_price, "silver_price")?;

    // Setup Config
    // Explicitly use Hanafi to ensure LowerOfTwo standard (Safer for the poor)
    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi) 
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);

    // Setup Business Assets
    let business = BusinessZakat::new()
        .cash(parse(&cash, "cash")?)
        .inventory(parse(&inventory, "inventory")?)
        .receivables(parse(&receivables, "receivables")?)
        .liabilities(parse(&liabilities, "liabilities")?)
        .hawl(true); // Assuming full hawl for simple API

    // Calculate
    match business.calculate_zakat(&config) {
        Ok(res) => Ok(DartZakatResult {
            zakat_due: res.zakat_due.to_f64().unwrap_or(0.0),
            is_payable: res.is_payable,
            nisab_threshold: res.nisab_threshold.to_f64().unwrap_or(0.0),
            wealth_amount: res.wealth_amount.to_f64().unwrap_or(0.0),
            limit_name: format!("{:?}", res.limit_name),
        }),
        Err(e) => Err(format!("Calculation failed: {:?}", e)),
    }
}

#[frb(sync)]
pub fn calculate_savings_zakat(
    cash_in_hand: String,
    bank_balance: String,
    gold_price: String,
    silver_price: String,
) -> Result<DartZakatResult, String> {
    let parse = |s: &str, name: &str| -> Result<Decimal, String> {
        Decimal::from_str(s).map_err(|e| format!("Invalid {}: {}", name, e))
    };

    let gold_price_dec = parse(&gold_price, "gold_price")?;
    let silver_price_dec = parse(&silver_price, "silver_price")?;

    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);

    let cash_val = parse(&cash_in_hand, "cash_in_hand")?;
    let bank_val = parse(&bank_balance, "bank_balance")?;

    // Savings are effectively "Cash Only" business assets or just pure wealth
    // We can use BusinessZakat::cash_only or manual calculation
    // Using BusinessZakat for consistency with "Urud al-Tijarah" / Wealth
    let wealth = BusinessZakat::new()
        .cash(cash_val + bank_val)
        .hawl(true);

    match wealth.calculate_zakat(&config) {
        Ok(res) => Ok(DartZakatResult {
            zakat_due: res.zakat_due.to_f64().unwrap_or(0.0),
            is_payable: res.is_payable,
            nisab_threshold: res.nisab_threshold.to_f64().unwrap_or(0.0),
            wealth_amount: res.wealth_amount.to_f64().unwrap_or(0.0),
            limit_name: format!("{:?}", res.limit_name),
        }),
        Err(e) => Err(format!("Calculation failed: {:?}", e)),
    }
}

#[frb(sync)]
pub fn get_nisab_thresholds(gold_price: String, silver_price: String) -> Result<(f64, f64), String> {
    let parse = |s: &str, name: &str| -> Result<Decimal, String> {
        Decimal::from_str(s).map_err(|e| format!("Invalid {}: {}", name, e))
    };

    let gold_price_dec = parse(&gold_price, "gold_price")?;
    let silver_price_dec = parse(&silver_price, "silver_price")?;

    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);
    
    let nisab_gold = config.get_nisab_threshold(WealthType::Gold).to_f64().unwrap_or(0.0);
    let nisab_silver = config.get_nisab_threshold(WealthType::Silver).to_f64().unwrap_or(0.0);
    
    Ok((nisab_gold, nisab_silver))
}
