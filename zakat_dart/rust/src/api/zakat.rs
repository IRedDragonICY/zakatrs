use flutter_rust_bridge::frb;
use zakat::prelude::*;
use rust_decimal::prelude::*;
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

// --- Native Portfolio (Stateful Asset Aggregation) ---

/// Result of portfolio calculation
pub struct DartPortfolioResult {
    pub total_zakat_due: FrbDecimal,
    pub total_assets: FrbDecimal,
    pub items_count: u32,
    pub is_payable: bool,
    pub nisab_threshold: FrbDecimal,
    pub status: String,
}

/// Portfolio item summary for display
pub struct DartPortfolioItem {
    pub id: String,
    pub label: String,
    pub wealth_type: String,
    pub zakat_due: FrbDecimal,
    pub is_payable: bool,
}

/// Stateful portfolio manager that aggregates multiple wealth types.
/// 
/// This enables "Dam' al-Amwal" (Wealth Aggregation) logic where multiple
/// wealth categories are combined for Nisab threshold checking and Zakat calculation.
pub struct NativePortfolio {
    portfolio: std::sync::Mutex<zakat::portfolio::ZakatPortfolio>,
    config: std::sync::Mutex<ZakatConfig>,
}

impl NativePortfolio {
    /// Creates a new NativePortfolio with the specified configuration.
    #[frb(sync)]
    pub fn new(gold_price: FrbDecimal, silver_price: FrbDecimal, madhab: String) -> Result<NativePortfolio> {
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

        Ok(NativePortfolio {
            portfolio: std::sync::Mutex::new(zakat::portfolio::ZakatPortfolio::default()),
            config: std::sync::Mutex::new(config),
        })
    }

    /// Adds a business asset to the portfolio.
    /// Returns the UUID of the added asset.
    #[frb(sync)]
    pub fn add_business(
        &self, 
        cash: FrbDecimal, 
        inventory: FrbDecimal, 
        receivables: FrbDecimal, 
        liabilities: FrbDecimal,
        label: String,
    ) -> Result<String> {
        let business = BusinessZakat::new()
            .cash(cash.0)
            .inventory(inventory.0)
            .receivables(receivables.0)
            .liabilities(liabilities.0)
            .label(label)
            .hawl(true);
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        // push() returns the UUID of the added item
        let id = portfolio.push(zakat::assets::PortfolioItem::Business(business));
        
        Ok(id.to_string())
    }

    /// Adds savings/cash asset to the portfolio.
    /// Returns the UUID of the added asset.
    #[frb(sync)]
    pub fn add_savings(
        &self, 
        cash_in_hand: FrbDecimal, 
        bank_balance: FrbDecimal,
        label: String,
    ) -> Result<String> {
        let total = cash_in_hand.0 + bank_balance.0;
        let business = BusinessZakat::new()
            .cash(total)
            .label(label)
            .hawl(true);
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        let id = portfolio.push(zakat::assets::PortfolioItem::Business(business));
        
        Ok(id.to_string())
    }

    /// Adds gold asset to the portfolio.
    /// Returns the UUID of the added asset.
    #[frb(sync)]
    pub fn add_gold(
        &self, 
        weight_grams: FrbDecimal,
        purity_karat: u32, // 1-24, e.g., 18 for 18K gold
        label: String,
    ) -> Result<String> {        
        let gold = zakat::maal::precious_metals::PreciousMetals::gold(weight_grams.0)
            .purity(purity_karat)
            .label(label)
            .hawl(true);
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        let id = portfolio.push(zakat::assets::PortfolioItem::PreciousMetals(gold));
        
        Ok(id.to_string())
    }

    /// Adds silver asset to the portfolio.
    /// Returns the UUID of the added asset.
    #[frb(sync)]
    pub fn add_silver(
        &self, 
        weight_grams: FrbDecimal,
        purity_per_thousand: u32, // e.g., 925 for sterling silver
        label: String,
    ) -> Result<String> {        
        let silver = zakat::maal::precious_metals::PreciousMetals::silver(weight_grams.0)
            .purity(purity_per_thousand)
            .label(label)
            .hawl(true);
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        let id = portfolio.push(zakat::assets::PortfolioItem::PreciousMetals(silver));
        
        Ok(id.to_string())
    }

    /// Removes an asset from the portfolio by its UUID.
    #[frb(sync)]
    pub fn remove_item(&self, id: String) -> Result<bool> {
        let uuid = id.parse::<uuid::Uuid>()
            .map_err(|e| anyhow::anyhow!("Invalid UUID: {}", e))?;
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        Ok(portfolio.remove(uuid).is_some())
    }

    /// Clears all items from the portfolio.
    #[frb(sync)]
    pub fn clear(&self) -> Result<()> {
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        // Replace with a new empty portfolio
        *portfolio = zakat::portfolio::ZakatPortfolio::default();
        Ok(())
    }

    /// Returns the number of items in the portfolio.
    #[frb(sync)]
    pub fn item_count(&self) -> Result<u32> {
        let portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        Ok(portfolio.get_items().len() as u32)
    }

    /// Calculates total Zakat due across all portfolio items.
    #[frb(sync)]
    pub fn calculate(&self) -> Result<DartPortfolioResult> {
        let portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        let config = self.config.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock config: {}", e))?;

        let result = portfolio.calculate_total(&*config);
        
        // Check for complete failure
        if result.status == zakat::portfolio::PortfolioStatus::Failed {
            return Err(anyhow::anyhow!("Portfolio calculation failed: {} items failed", result.items_failed));
        }

        Ok(DartPortfolioResult {
            total_zakat_due: FrbDecimal(result.total_zakat_due),
            total_assets: FrbDecimal(result.total_assets),
            items_count: result.successes().len() as u32,
            is_payable: result.total_zakat_due > Decimal::ZERO,
            nisab_threshold: FrbDecimal(config.get_monetary_nisab_threshold()),
            status: format!("{:?}", result.status),
        })
    }

    /// Gets individual results for each portfolio item.
    #[frb(sync)]
    pub fn get_item_results(&self) -> Result<Vec<DartPortfolioItem>> {
        let portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
        let config = self.config.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock config: {}", e))?;

        let result = portfolio.calculate_total(&*config);

        let items: Vec<DartPortfolioItem> = result.successes().iter()
            .map(|details| DartPortfolioItem {
                id: "".to_string(), // ID not preserved in ZakatDetails currently
                label: details.label.clone().unwrap_or_default(),
                wealth_type: format!("{:?}", details.wealth_type),
                zakat_due: FrbDecimal(details.zakat_due),
                is_payable: details.is_payable,
            })
            .collect();

        Ok(items)
    }
}
