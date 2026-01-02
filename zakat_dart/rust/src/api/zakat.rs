use flutter_rust_bridge::frb;
use zakat::prelude::*;
use rust_decimal::prelude::*;
use std::str::FromStr;
use anyhow::Result;

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

#[frb(sync)]
pub fn validate_input_string(s: String) -> bool {
    zakat::inputs::validate_numeric_format(&s)
}

// --- Data Types ---

/// A wrapper around rust_decimal::Decimal to be used across the FFI boundary.
/// This ensures type safety and prevents floating point errors.
#[derive(Debug, Clone)]
pub struct FrbDecimal(Decimal);

impl FrbDecimal {
    #[frb(sync)]
    pub fn from_string(s: String) -> Result<Self> {
        let d = zakat::inputs::IntoZakatDecimal::into_zakat_decimal(s.as_str())
            .map_err(|e| anyhow::anyhow!("Invalid decimal format '{}': {}", s, e))?;
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

// --- Zakat Manager ---

pub struct ZakatManager {
    config: ZakatConfig,
}

impl ZakatManager {
    #[frb(sync)]
    pub fn new(gold_price: FrbDecimal, silver_price: FrbDecimal, madhab: String) -> Result<ZakatManager> {
        let madhab_enum = match madhab.to_lowercase().as_str() {
            "hanafi" => Madhab::Hanafi,
            "shafi" => Madhab::Shafi,
            "maliki" => Madhab::Maliki,
            "hanbali" => Madhab::Hanbali,
            _ => return Err(anyhow::anyhow!("Invalid madhab: {}", madhab)),
        };

        let config = ZakatConfig::new()
            .with_madhab(madhab_enum)
            .with_gold_price(gold_price.0)
            .with_silver_price(silver_price.0);

        Ok(ZakatManager { config })
    }

    #[frb(sync)]
    pub fn calculate_business(
        &self, 
        cash: FrbDecimal, 
        inventory: FrbDecimal, 
        receivables: FrbDecimal, 
        liabilities: FrbDecimal
    ) -> Result<DartZakatResult> {
        // Setup Business Assets
        // Note: simple.rs used hawl(true) hardcoded, so we continue that valid assumption for now 
        // as the user didn't specify hawl in the new API.
        let business = BusinessZakat::new()
            .cash(cash.0)
            .inventory(inventory.0)
            .receivables(receivables.0)
            .liabilities(liabilities.0)
            .hawl(true);

        match business.calculate_zakat(&self.config) {
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
    pub fn calculate_savings(
        &self, 
        cash_in_hand: FrbDecimal, 
        bank_balance: FrbDecimal
    ) -> Result<DartZakatResult> {
        let cash_val = cash_in_hand.0;
        let bank_val = bank_balance.0;

        // Using BusinessZakat for consistency with previous simple.rs implementation for general monetary assets
        let wealth = BusinessZakat::new()
            .cash(cash_val + bank_val)
            .hawl(true);

        match wealth.calculate_zakat(&self.config) {
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
    pub fn get_nisab_thresholds(&self) -> Result<(FrbDecimal, FrbDecimal)> {
        // Accessing config logic directly as in simple.rs
        // Note: verify if get_nisab_gold_grams is public or if we need to recalculate.
        // simple.rs used: config.gold_price_per_gram * config.get_nisab_gold_grams()
        // Assuming get_nisab_gold_grams() follows the Madhab set in config.
        
        // However, ZakatConfig in crate might not expose get_nisab_gold_grams publicly inside the crate 
        // if it's not in the prelude or public API. 
        // `simple.rs` used: `let nisab_gold = config.gold_price_per_gram * config.get_nisab_gold_grams();`
        // So it seems it is available.
        
        let nisab_gold = self.config.gold_price_per_gram * self.config.get_nisab_gold_grams();
        let nisab_silver = self.config.silver_price_per_gram * self.config.get_nisab_silver_grams();
        
        Ok((FrbDecimal(nisab_gold), FrbDecimal(nisab_silver)))
    }
}
