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
#[derive(Debug, Serialize, Deserialize)]
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
            PortfolioStatus::Partial => Err(ZakatError::CalculationError(
                format!("Portfolio calculation incomplete. {}/{} items failed.", self.items_failed, self.items_attempted), 
                Some("Portfolio".to_string())
            )),
            PortfolioStatus::Failed => Err(ZakatError::CalculationError(
                "Portfolio calculation failed completely.".to_string(), 
                Some("Portfolio".to_string())
            )),
        }
    }
}

pub struct ZakatPortfolio {
    calculators: Vec<Box<dyn CalculateZakat + Send + Sync>>,
}

impl ZakatPortfolio {
    pub fn new() -> Self {
        Self {
            calculators: Vec::new(),
        }
    }
}

impl Default for ZakatPortfolio {
    fn default() -> Self {
        Self::new()
    }
}

impl ZakatPortfolio {
    #[allow(clippy::should_implement_trait)]
    pub fn add<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T) -> Self {
         self.calculators.push(Box::new(calculator));
         self
    }

    /// Adds an asset and returns the portfolio along with the asset's UUID.
    /// Useful for tracking the asset for later updates/removals.
    pub fn add_with_id<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T) -> (Self, Uuid) {
        let id = calculator.get_id();
        self.calculators.push(Box::new(calculator));
        (self, id)
    }

    /// Adds an asset to a mutable reference and returns its UUID.
    pub fn push<T: CalculateZakat + Send + Sync + 'static>(&mut self, calculator: T) -> Uuid {
        let id = calculator.get_id();
        self.calculators.push(Box::new(calculator));
        id
    }

    /// Removes an asset by its UUID. Returns the removed calculator if found.
    pub fn remove(&mut self, id: Uuid) -> Option<Box<dyn CalculateZakat + Send + Sync>> {
        if let Some(pos) = self.calculators.iter().position(|c| c.get_id() == id) {
            Some(self.calculators.remove(pos))
        } else {
            None
        }
    }

    /// Replaces an asset by its UUID.
    pub fn replace<T: CalculateZakat + Send + Sync + 'static>(&mut self, id: Uuid, new_calculator: T) -> Result<(), ZakatError> {
        if let Some(pos) = self.calculators.iter().position(|c| c.get_id() == id) {
            self.calculators[pos] = Box::new(new_calculator);
            Ok(())
        } else {
            Err(ZakatError::InvalidInput(format!("Asset with ID {} not found", id), None))
        }
    }

    /// Gets a reference to an asset by ID.
    pub fn get(&self, id: Uuid) -> Option<&(dyn CalculateZakat + Send + Sync)> {
        self.calculators.iter().find(|c| c.get_id() == id).map(|b| b.as_ref())
    }

    /// Calculates Zakat for all assets in the portfolio.
    pub fn calculate_total(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
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
                items_attempted: self.calculators.len(),
                items_failed: self.calculators.len(),
            };
        }

        let mut results = Vec::new();

        // 1. Initial calculation for all assets
        for (index, item) in self.calculators.iter().enumerate() {
            match item.calculate_zakat(config) {
                Ok(detail) => results.push(PortfolioItemResult::Success {
                     asset_id: item.get_id(),
                     details: detail 
                }),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = item.get_label() {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    err = err.with_source(source.clone());
                    results.push(PortfolioItemResult::Failure {
                        asset_id: item.get_id(),
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
                items_attempted: self.calculators.len(),
                items_failed: self.calculators.len(),
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
                         // Calculator removed? Keep the error or mark as removed?
                         // If removed, we probably shouldn't include it in the new result?
                         // Or preserve the old error. preserving old error is safer.
                         new_results.push(result.clone());
                     }
                }
            }
        }
        
        aggregate_and_summarize(new_results, config)
    }
}

#[cfg(feature = "async")]
pub struct AsyncZakatPortfolio {
    calculators: Vec<Box<dyn AsyncCalculateZakat>>,
}

#[cfg(feature = "async")]
impl AsyncZakatPortfolio {
    pub fn new() -> Self {
        Self {
            calculators: Vec::new(),
        }
    }
    
    #[allow(clippy::should_implement_trait)]
    pub fn add<T: AsyncCalculateZakat + 'static>(mut self, calculator: T) -> Self {
         self.calculators.push(Box::new(calculator));
         self
    }
    
    // Helper methods for specific calculator types can be added here.
    
    /// Calculates Zakat asynchronously for all assets in the portfolio.
    pub async fn calculate_total_async(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
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
                items_attempted: self.calculators.len(),
                items_failed: self.calculators.len(),
            };
        }

        let mut results = Vec::new();

        for (index, item) in self.calculators.iter().enumerate() {
            match item.calculate_zakat_async(config).await {
                Ok(detail) => results.push(PortfolioItemResult::Success {
                     asset_id: item.get_id(),
                     details: detail 
                }),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = item.get_label() {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    err = err.with_source(source.clone());
                    results.push(PortfolioItemResult::Failure {
                        asset_id: item.get_id(),
                        source,
                        error: err,
                    });
                },
            }
        }
        
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
        let standard_rate = rust_decimal_macros::dec!(0.025);

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
                        "Aggregated Monetary Wealth > Nisab -> Payable (Dam' al-Amwal)"
                    ));
                    details.calculation_trace.push(crate::types::CalculationStep::result(
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
