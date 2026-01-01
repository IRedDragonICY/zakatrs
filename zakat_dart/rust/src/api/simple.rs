use flutter_rust_bridge::frb;
use zakat::prelude::*;
use rust_decimal::prelude::*;
use std::str::FromStr;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

// --- Data Types ---

/// A wrapper around rust_decimal::Decimal to be used across the FFI boundary.
/// This ensures type safety and prevents floating point errors.
#[derive(Debug, Clone)]
pub struct FrbDecimal(Decimal);

impl FrbDecimal {
    #[frb(sync)]
    pub fn from_string(s: String) -> anyhow::Result<Self> {
        let d = Decimal::from_str(&s).map_err(|e| anyhow::anyhow!("Invalid decimal: {}", e))?;
        Ok(Self(d))
    }

    #[frb(sync)]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    
    // Allow using as rudimentary types if needed, but prefer string for precision
    #[frb(sync)]
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap_or(0.0)
    }
}

pub struct DartZakatResult {
    pub zakat_due: FrbDecimal,
    pub is_payable: bool,
    pub nisab_threshold: FrbDecimal,
    pub wealth_amount: FrbDecimal,
    pub limit_name: String,
}

// --- API Methods ---

#[frb(sync)]
pub fn calculate_business_zakat(
    cash: FrbDecimal,
    inventory: FrbDecimal,
    receivables: FrbDecimal,
    liabilities: FrbDecimal,
    gold_price: FrbDecimal,
    silver_price: FrbDecimal,
) -> anyhow::Result<DartZakatResult> {
    
    // Unwrap inner decimals
    let gold_price_dec = gold_price.0;
    let silver_price_dec = silver_price.0;

    // Setup Config
    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi) 
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);

    // Setup Business Assets
    let business = BusinessZakat::new()
        .cash(cash.0)
        .inventory(inventory.0)
        .receivables(receivables.0)
        .liabilities(liabilities.0)
        .hawl(true);

    // Calculate
    match business.calculate_zakat(&config) {
        Ok(res) => Ok(DartZakatResult {
            zakat_due: FrbDecimal(res.zakat_due),
            is_payable: res.is_payable,
            nisab_threshold: FrbDecimal(res.nisab_threshold),
            wealth_amount: FrbDecimal(res.net_assets),
            limit_name: format!("{:?}", res.wealth_type),
        }),
        Err(e) => Err(anyhow::anyhow!("Calculation failed: {:?}", e)),
    }
}

#[frb(sync)]
pub fn calculate_savings_zakat(
    cash_in_hand: FrbDecimal,
    bank_balance: FrbDecimal,
    gold_price: FrbDecimal,
    silver_price: FrbDecimal,
) -> anyhow::Result<DartZakatResult> {
    
    let gold_price_dec = gold_price.0;
    let silver_price_dec = silver_price.0;

    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);

    let cash_val = cash_in_hand.0;
    let bank_val = bank_balance.0;

    // Using BusinessZakat for consistency
    let wealth = BusinessZakat::new()
        .cash(cash_val + bank_val)
        .hawl(true);

    match wealth.calculate_zakat(&config) {
        Ok(res) => Ok(DartZakatResult {
            zakat_due: FrbDecimal(res.zakat_due),
            is_payable: res.is_payable,
            nisab_threshold: FrbDecimal(res.nisab_threshold),
            wealth_amount: FrbDecimal(res.net_assets),
            limit_name: format!("{:?}", res.wealth_type),
        }),
        Err(e) => Err(anyhow::anyhow!("Calculation failed: {:?}", e)),
    }
}

#[frb(sync)]
pub fn get_nisab_thresholds(
    gold_price: FrbDecimal, 
    silver_price: FrbDecimal
) -> anyhow::Result<(FrbDecimal, FrbDecimal)> {
    
    let gold_price_dec = gold_price.0;
    let silver_price_dec = silver_price.0;

    let config = ZakatConfig::new()
        .with_madhab(Madhab::Hanafi)
        .with_gold_price(gold_price_dec)
        .with_silver_price(silver_price_dec);
    
    let nisab_gold = config.gold_price_per_gram * config.get_nisab_gold_grams();
    let nisab_silver = config.silver_price_per_gram * config.get_nisab_silver_grams();
    
    Ok((FrbDecimal(nisab_gold), FrbDecimal(nisab_silver)))
}
