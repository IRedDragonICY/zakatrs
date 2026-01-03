/// Portfolio and Manager API for Dart.
///
/// This module provides stateful portfolio management for aggregating
/// multiple wealth types and calculating total Zakat.

use flutter_rust_bridge::frb;
use zakat::prelude::*;
use rust_decimal::prelude::*;
use anyhow::Result;

// Re-export shared types for convenience
pub use super::types::{FrbDecimal, DartZakatConfig, DartZakatResult};

#[frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

/// Validate if a string is a valid decimal format.
#[frb(sync)]
pub fn validate_input_string(s: String) -> bool {
    zakat::inputs::validate_numeric_format(&s)
}

// ============================================================================
// Portfolio Result Types
// ============================================================================

/// Result of portfolio calculation.
pub struct DartPortfolioResult {
    pub total_zakat_due: FrbDecimal,
    pub total_assets: FrbDecimal,
    pub items_count: u32,
    pub is_payable: bool,
    pub nisab_threshold: FrbDecimal,
    pub status: String,
}

/// Portfolio item summary for display.
pub struct DartPortfolioItem {
    pub id: String,
    pub label: String,
    pub wealth_type: String,
    pub zakat_due: FrbDecimal,
    pub is_payable: bool,
}

// ============================================================================
// Stateful Portfolio Manager
// ============================================================================

/// Stateful portfolio manager that aggregates multiple wealth types.
/// 
/// This enables "Dam' al-Amwal" (Wealth Aggregation) logic where multiple
/// wealth categories are combined for Nisab threshold checking and Zakat calculation.
///
/// # Example (Dart)
/// ```dart
/// final portfolio = NativePortfolio(
///   goldPrice: FrbDecimal.fromString("250"),
///   silverPrice: FrbDecimal.fromString("3"),
///   madhab: "hanafi",
/// );
///
/// portfolio.addBusiness(
///   cash: FrbDecimal.fromString("50000"),
///   inventory: FrbDecimal.fromString("25000"),
///   receivables: FrbDecimal.zero(),
///   liabilities: FrbDecimal.zero(),
///   label: "Main Store",
/// );
///
/// final result = portfolio.calculate();
/// ```
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
            "shafi" | "shafii" => Madhab::Shafi,
            "maliki" => Madhab::Maliki,
            "hanbali" => Madhab::Hanbali,
            _ => return Err(anyhow::anyhow!("Invalid madhab: {}", madhab)),
        };

        let config = ZakatConfig::new()
            .with_madhab(madhab_enum)
            .with_gold_price(gold_price.value)
            .with_silver_price(silver_price.value);

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
            .cash(cash.value)
            .inventory(inventory.value)
            .receivables(receivables.value)
            .liabilities(liabilities.value)
            .label(label)
            .hawl(true);
        
        let mut portfolio = self.portfolio.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock portfolio: {}", e))?;
        
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
        let total = cash_in_hand.value + bank_balance.value;
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
        purity_karat: u32,
        label: String,
    ) -> Result<String> {        
        let gold = zakat::maal::precious_metals::PreciousMetals::gold(weight_grams.value)
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
        purity_per_thousand: u32,
        label: String,
    ) -> Result<String> {        
        let silver = zakat::maal::precious_metals::PreciousMetals::silver(weight_grams.value)
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
        
        if result.status == zakat::portfolio::PortfolioStatus::Failed {
            return Err(anyhow::anyhow!("Portfolio calculation failed: {} items failed", result.items_failed));
        }

        Ok(DartPortfolioResult {
            total_zakat_due: FrbDecimal { value: result.total_zakat_due },
            total_assets: FrbDecimal { value: result.total_assets },
            items_count: result.successes().len() as u32,
            is_payable: result.total_zakat_due > Decimal::ZERO,
            nisab_threshold: FrbDecimal { value: config.get_monetary_nisab_threshold() },
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
                id: "".to_string(),
                label: details.label.clone().unwrap_or_default(),
                wealth_type: format!("{:?}", details.wealth_type),
                zakat_due: FrbDecimal { value: details.zakat_due },
                is_payable: details.is_payable,
            })
            .collect();

        Ok(items)
    }
}
