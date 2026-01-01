//! # Fiqh Compliance: Portfolio Aggregation
//!
//! ## Principle: Dam' al-Amwal (Joining Wealth)
//! - Implements the **Hanafi** and Majority view that Gold, Silver, Cash, and Trade Goods are of a single genus (*Thamaniyyah*) and must be combined to reach the Nisab.
//! - **Benefit**: This ensures the poor receive their due from wealth that would otherwise be exempt if split (*Anfa' lil-fuqara*).

use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::traits::CalculateZakat;
#[cfg(feature = "async")]
use crate::traits::AsyncCalculateZakat;
use crate::types::{ZakatDetails, ZakatError};
use tracing::{instrument, info, warn};

/// Individual result for an asset in the portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortfolioItemResult {
    /// Calculation succeeded
    Success {
        asset_id: Uuid,
        details: ZakatDetails,
    },
    /// Calculation failed
    Failure {
        asset_id: Uuid,
        source: String, // Label or Index
        error: ZakatError,
    },
}

impl PortfolioItemResult {
    pub fn asset_id(&self) -> Uuid {
        match self {
            Self::Success { asset_id, .. } => *asset_id,
            Self::Failure { asset_id, .. } => *asset_id,
        }
    }
}

/// Status of the portfolio calculation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortfolioStatus {
    /// All items calculated successfully.
    Complete,
    /// Some items failed, but others succeeded. Result contains partial totals.
    Partial,
    /// All items failed.
    Failed,
}

/// Result of a portfolio calculation, including successes and partial failures.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioResult {
    pub status: PortfolioStatus,
    pub results: Vec<PortfolioItemResult>,
    pub total_assets: Decimal,
    pub total_zakat_due: Decimal,
    pub items_attempted: usize,
    pub items_failed: usize,
}

impl PortfolioResult {
    /// Returns a list of failed calculations.
    pub fn failures(&self) -> Vec<&PortfolioItemResult> {
        self.results.iter().filter(|r| matches!(r, PortfolioItemResult::Failure { .. })).collect()
    }

    /// Returns a list of successful calculation details.
    pub fn successes(&self) -> Vec<&ZakatDetails> {
        self.results.iter().filter_map(|r| match r {
            PortfolioItemResult::Success { details, .. } => Some(details),
            _ => None
        }).collect()
    }

    /// Returns true if there were no failures.
    pub fn is_clean(&self) -> bool {
        self.status == PortfolioStatus::Complete
    }
    
    /// Returns the result if Complete, otherwise returns an error describing the failure(s).
    pub fn expect_complete(self) -> Result<Self, ZakatError> {
        match self.status {
            PortfolioStatus::Complete => Ok(self),
            PortfolioStatus::Partial => Err(ZakatError::CalculationError {
                reason: format!("Portfolio calculation incomplete. {}/{} items failed.", self.items_failed, self.items_attempted), 
                source_label: Some("Portfolio".to_string()),
                asset_id: None,
            }),
            PortfolioStatus::Failed => Err(ZakatError::CalculationError {
                reason: "Portfolio calculation failed completely.".to_string(), 
                source_label: Some("Portfolio".to_string()),
                asset_id: None,
            }),
        }
    }
}

use crate::assets::PortfolioItem;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZakatPortfolio {
    items: Vec<PortfolioItem>,
}

impl ZakatPortfolio {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(BusinessZakat::cash_only(...)) instead")]
    pub fn add_business<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::business::BusinessZakat) -> crate::maal::business::BusinessZakat {
        let asset = f(crate::maal::business::BusinessZakat::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(AgricultureAssets::new()...) instead")]
    pub fn add_agriculture<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::agriculture::AgricultureAssets) -> crate::maal::agriculture::AgricultureAssets {
        let asset = f(crate::maal::agriculture::AgricultureAssets::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(LivestockAssets::new()...) instead")]
    pub fn add_livestock<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::livestock::LivestockAssets) -> crate::maal::livestock::LivestockAssets {
        let asset = f(crate::maal::livestock::LivestockAssets::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(IncomeZakatCalculator::from_salary(...)) instead")]
    pub fn add_income<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::income::IncomeZakatCalculator) -> crate::maal::income::IncomeZakatCalculator {
        let asset = f(crate::maal::income::IncomeZakatCalculator::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(InvestmentAssets::stock(...)) instead")]
    pub fn add_investment<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::investments::InvestmentAssets) -> crate::maal::investments::InvestmentAssets {
        let asset = f(crate::maal::investments::InvestmentAssets::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(MiningAssets::new()...) instead")]
    pub fn add_mining<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::mining::MiningAssets) -> crate::maal::mining::MiningAssets {
        let asset = f(crate::maal::mining::MiningAssets::new());
        self.add(asset)
    }

    #[deprecated(since = "0.2.0", note = "Use generic .add(PreciousMetals::gold(...)) instead")]
    pub fn add_precious_metals<F>(self, f: F) -> Self 
    where F: FnOnce(crate::maal::precious_metals::PreciousMetals) -> crate::maal::precious_metals::PreciousMetals {
        let asset = f(crate::maal::precious_metals::PreciousMetals::new());
        self.add(asset)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add<T: Into<PortfolioItem>>(mut self, item: T) -> Self {
         self.items.push(item.into());
         self
    }

    /// Adds an asset and returns the portfolio along with the asset's UUID.
    /// Useful for tracking the asset for later updates/removals.
    pub fn add_with_id<T: Into<PortfolioItem>>(mut self, item: T) -> (Self, Uuid) {
        let item: PortfolioItem = item.into();
        let id = CalculateZakat::get_id(&item);
        self.items.push(item);
        (self, id)
    }

    /// Adds an asset to a mutable reference and returns its UUID.
    pub fn push<T: Into<PortfolioItem>>(&mut self, item: T) -> Uuid {
        let item: PortfolioItem = item.into();
        let id = CalculateZakat::get_id(&item);
        self.items.push(item);
        id
    }

    /// Removes an asset by its UUID. Returns the removed item if found.
    pub fn remove(&mut self, id: Uuid) -> Option<PortfolioItem> {
        if let Some(pos) = self.items.iter().position(|c| CalculateZakat::get_id(c) == id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }

    /// Replaces an asset by its UUID.
    pub fn replace<T: Into<PortfolioItem>>(&mut self, id: Uuid, new_item: T) -> Result<(), ZakatError> {
        if let Some(pos) = self.items.iter().position(|c| CalculateZakat::get_id(c) == id) {
            self.items[pos] = new_item.into();
            Ok(())
        } else {
            Err(ZakatError::InvalidInput {
                field: "asset_id".to_string(),
                value: id.to_string(),
                reason: "Asset with ID not found".to_string(),
                source_label: None,
                asset_id: None,
            })
        }
    }

    /// Gets a reference to an asset by ID.
    pub fn get(&self, id: Uuid) -> Option<&PortfolioItem> {
        self.items.iter().find(|c| CalculateZakat::get_id(*c) == id)
    }

    /// Gets a mutable reference to an asset by ID.
    ///
    /// This allows users to modify an existing asset in the portfolio directly:
    /// ```rust,ignore
    /// portfolio.get_mut(id).map(|item| { /* modify item */ });
    /// ```
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut PortfolioItem> {
        self.items.iter_mut().find(|c| CalculateZakat::get_id(*c) == id)
    }

    /// Returns a slice of all items in the portfolio.
    pub fn get_items(&self) -> &[PortfolioItem] {
        &self.items
    }

    /// Merges another portfolio into this one.
    ///
    /// Consumes the `other` portfolio and moves all its items into `self`.
    pub fn merge(mut self, other: ZakatPortfolio) -> Self {
        self.items.extend(other.items);
        self
    }

    /// Calculates Zakat for all assets in the portfolio.
    #[instrument(skip(self, config), fields(items_count = self.items.len()))]
    pub fn calculate_total(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
        info!("Starting portfolio calculation");
        // Fail Fast: Validate config before processing any items
        if let Err(e) = config.validate() {
            return PortfolioResult {
                status: PortfolioStatus::Failed,
                results: vec![PortfolioItemResult::Failure {
                    asset_id: Uuid::nil(), // No specific asset
                    source: "Configuration".to_string(),
                    error: e,
                }],
                total_assets: Decimal::ZERO,
                total_zakat_due: Decimal::ZERO,
                items_attempted: self.items.len(),
                items_failed: self.items.len(),
            };
        }

        let mut results = Vec::new();

        // 1. Initial calculation for all assets
        for (index, item) in self.items.iter().enumerate() {
            match item.calculate_zakat(config) {
                Ok(detail) => results.push(PortfolioItemResult::Success {
                     asset_id: CalculateZakat::get_id(item),
                     details: detail 
                }),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = CalculateZakat::get_label(item) {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    warn!(error = ?err, source = %source, "Asset calculation failed");
                    err = err.with_source(source.clone());
                    results.push(PortfolioItemResult::Failure {
                        asset_id: CalculateZakat::get_id(item),
                        source,
                        error: err,
                    });
                },
            }
        }

        aggregate_and_summarize(results, config)
    }

    /// Retries failed items from a previous calculation using a new (presumably fixed) configuration.
    pub fn retry_failures(&self, previous_result: &PortfolioResult, config: &crate::config::ZakatConfig) -> PortfolioResult {
        // If config is still invalid, fail immediately
        if let Err(e) = config.validate() {
             return PortfolioResult {
                status: PortfolioStatus::Failed,
                results: vec![PortfolioItemResult::Failure {
                    asset_id: Uuid::nil(),
                    source: "Configuration".to_string(),
                    error: e,
                }],
                total_assets: Decimal::ZERO,
                total_zakat_due: Decimal::ZERO,
                items_attempted: self.items.len(),
                items_failed: self.items.len(),
            };
        }

        let mut new_results = Vec::with_capacity(previous_result.results.len());
        
        // We iterate over previous results and retry ONLY the failures.
        // We find the corresponding calculator by ID.
        
        for result in &previous_result.results {
            match result {
                PortfolioItemResult::Success { .. } => {
                    new_results.push(result.clone());
                },
                PortfolioItemResult::Failure { asset_id, source, error: _ } => {
                     // Try to find the calculator with this ID
                     if let Some(calc) = self.get(*asset_id) {
                         match calc.calculate_zakat(config) {
                             Ok(detail) => new_results.push(PortfolioItemResult::Success { 
                                 asset_id: *asset_id, 
                                 details: detail 
                             }),
                             Err(new_err) => {
                                 let mut e = new_err;
                                 e = e.with_source(source.clone());
                                 new_results.push(PortfolioItemResult::Failure {
                                     asset_id: *asset_id,
                                     source: source.clone(),
                                     error: e,
                                 });
                             }
                         }
                     } else {
                         // If the calculator was removed, we preserve the original error to maintain history.
                         new_results.push(result.clone());
                     }
                }
            }
        }
        
        aggregate_and_summarize(new_results, config)
    }
}

#[cfg(feature = "async")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncZakatPortfolio {
    items: Vec<PortfolioItem>,
}

#[cfg(feature = "async")]
impl AsyncZakatPortfolio {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn add<T: Into<PortfolioItem>>(mut self, item: T) -> Self {
         self.items.push(item.into());
         self
    }
    
    // Helper methods for specific calculator types can be added here.
    
    /// Calculates Zakat asynchronously for all assets in the portfolio.
    #[instrument(skip(self, config), fields(items_count = self.items.len()))]
    pub async fn calculate_total_async(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
        info!("Starting async portfolio calculation");
        // Fail Fast: Validate config before processing any items
        if let Err(e) = config.validate() {
            return PortfolioResult {
                status: PortfolioStatus::Failed,
                results: vec![PortfolioItemResult::Failure {
                    asset_id: Uuid::nil(),
                    source: "Configuration".to_string(),
                    error: e,
                }],
                total_assets: Decimal::ZERO,
                total_zakat_due: Decimal::ZERO,
                items_attempted: self.items.len(),
                items_failed: self.items.len(),
            };
        }

        use futures::stream::StreamExt;
        let mut futures = futures::stream::FuturesUnordered::new();

        for (index, item) in self.items.iter().enumerate() {
            let config = config.clone();
            let item = item.clone();
            
            futures.push(async move {
                let res = item.calculate_zakat_async(&config).await;
                (index, item, res)
            });
        }

        let mut temp_results = Vec::with_capacity(self.items.len());

        while let Some((index, item, res)) = futures.next().await {
            match res {
                Ok(detail) => temp_results.push((index, PortfolioItemResult::Success {
                     asset_id: CalculateZakat::get_id(&item),
                     details: detail 
                })),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = CalculateZakat::get_label(&item) {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    err = err.with_source(source.clone());
                    temp_results.push((index, PortfolioItemResult::Failure {
                        asset_id: CalculateZakat::get_id(&item),
                        source,
                        error: err,
                    }));
                },
            }
        }
        
        // Restore order
        temp_results.sort_by_key(|(i, _)| *i);
        let results = temp_results.into_iter().map(|(_, r)| r).collect();
        
        aggregate_and_summarize(results, config)
    }
}

#[cfg(feature = "async")]
impl Default for AsyncZakatPortfolio {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared logic to aggregate results and apply Dam' al-Amwal (Wealth Aggregation).
#[allow(clippy::collapsible_if)]
fn aggregate_and_summarize(mut results: Vec<PortfolioItemResult>, config: &crate::config::ZakatConfig) -> PortfolioResult {
    // 2. Aggregation Logic (Dam' al-Amwal)
    // Filter monetary assets (Gold, Silver, Cash, Business, Investments) from SUCCESSFUL results
    let mut monetary_net_assets = Decimal::ZERO;
    let mut monetary_indices = Vec::new();

    for (i, result) in results.iter().enumerate() {
        if let PortfolioItemResult::Success { details, .. } = result {
             if details.wealth_type.is_monetary() {
                monetary_net_assets += details.net_assets;
                monetary_indices.push(i);
             }
        }
    }
    
    // Check against the global monetary Nisab
    let global_nisab = config.get_monetary_nisab_threshold();
    
    if monetary_net_assets >= global_nisab && monetary_net_assets > Decimal::ZERO {
        let standard_rate = config.strategy.get_rules().trade_goods_rate;

        for i in monetary_indices {
            // We need to mutate the result.
            if let Some(PortfolioItemResult::Success { details, .. }) = results.get_mut(i) {
                if !details.is_payable {
                    details.is_payable = true;
                    details.status_reason = Some("Payable via Aggregation (Dam' al-Amwal)".to_string());
                    
                    // Recalculate zakat due
                    if details.net_assets > Decimal::ZERO {
                        details.zakat_due = details.net_assets * standard_rate;
                    }
                    
                    // Add trace step explaining aggregation
                    details.calculation_trace.push(crate::types::CalculationStep::info(
                        "info-aggregation-payable",
                        "Aggregated Monetary Wealth > Nisab -> Payable (Dam' al-Amwal)"
                    ));
                    details.calculation_trace.push(crate::types::CalculationStep::result(
                        "step-recalculated-zakat",
                        "Recalculated Zakat Due", details.zakat_due
                    ));
                }
            }
        }
    }

    // 3. Final Summation (only successes)
    let mut total_assets = Decimal::ZERO;
    let mut total_zakat_due = Decimal::ZERO;
    let items_attempted = results.len();
    let items_failed = results.iter().filter(|r| matches!(r, PortfolioItemResult::Failure { .. })).count();

    for result in &results {
        if let PortfolioItemResult::Success { details, .. } = result {
            total_assets += details.total_assets;
            total_zakat_due += details.zakat_due;
        }
    }

    let status = if items_failed == 0 {
        PortfolioStatus::Complete
    } else if items_failed == items_attempted {
        PortfolioStatus::Failed
    } else {
        PortfolioStatus::Partial
    };

    PortfolioResult {
        status,
        results,
        total_assets,
        total_zakat_due,
        items_attempted,
        items_failed,
    }
}
