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
    cash: f64,
    inventory: f64,
    receivables: f64,
    liabilities: f64,
    gold_price: f64,
    silver_price: f64,
) -> Result<DartZakatResult, String> {
    // Setup Config
    // Explicitly use Hanafi to ensure LowerOfTwo standard (Safer for the poor)
    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi) 
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);

    // Setup Business Assets
    let business = BusinessZakat::new()
        .cash(cash)
        .inventory(inventory)
        .receivables(receivables)
        .liabilities(liabilities)
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
    cash_in_hand: f64,
    bank_balance: f64,
    gold_price: f64,
    silver_price: f64,
) -> Result<DartZakatResult, String> {
    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);

    // Savings are effectively "Cash Only" business assets or just pure wealth
    // We can use BusinessZakat::cash_only or manual calculation
    // Using BusinessZakat for consistency with "Urud al-Tijarah" / Wealth
    let wealth = BusinessZakat::new()
        .cash(cash_in_hand + bank_balance)
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
pub fn get_nisab_thresholds(gold_price: f64, silver_price: f64) -> (f64, f64) {
    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price)
        .with_silver_price(silver_price);
    
    let nisab_gold = config.get_nisab_threshold(WealthType::Gold).to_f64().unwrap_or(0.0);
    let nisab_silver = config.get_nisab_threshold(WealthType::Silver).to_f64().unwrap_or(0.0);
    
    (nisab_gold, nisab_silver)
}
