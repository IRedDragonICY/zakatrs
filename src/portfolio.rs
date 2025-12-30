//! # Fiqh Compliance: Portfolio Aggregation
//!
//! ## Principle: Dam' al-Amwal (Joining Wealth)
//! - Implements the **Hanafi** and Majority view that Gold, Silver, Cash, and Trade Goods are of a single genus (*Thamaniyyah*) and must be combined to reach the Nisab.
//! - **Benefit**: This ensures the poor receive their due from wealth that would otherwise be exempt if split (*Anfa' lil-fuqara*).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::traits::CalculateZakat;
#[cfg(feature = "async")]
use crate::traits::AsyncCalculateZakat;
use crate::types::{ZakatDetails, ZakatError};

/// Individual result for an asset in the portfolio.
#[derive(Debug, Serialize, Deserialize)]
pub enum PortfolioItemResult {
    /// Calculation succeeded
    Success(ZakatDetails),
    /// Calculation failed
    Failure {
        source: String, // Label or Index
        error: ZakatError,
    },
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
            PortfolioItemResult::Success(d) => Some(d),
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

// Wrapper handles storing the item-specific debt context if provided.

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

    // Helper methods for specific calculator types can be added here.
    
    /// Calculates Zakat for all assets in the portfolio.
    ///
    /// # Aggregation Logic (Dam' al-Amwal)
    ///
    /// This method implements the Fiqh principle of **"Dam' al-Amwal"** (ضم الأموال),
    /// which means "combining wealth" for Nisab purposes.
    ///
    /// ## How It Works
    /// 1. First, each asset is calculated individually against its own Nisab.
    /// 2. Then, all **monetary assets** (Gold, Silver, Cash, Business, Investments)
    ///    are aggregated and checked against a single monetary Nisab threshold.
    /// 3. If the **combined total** meets the Nisab, all monetary assets become
    ///    payable even if they individually fell below their thresholds.
    ///
    /// ## Fiqh Basis
    /// This follows the scholarly position that similar wealth types should be
    /// combined when determining Zakat eligibility, benefiting the recipients
    /// of Zakat by ensuring wealth that collectively exceeds Nisab is not exempt.
    pub fn calculate_total(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
        let mut results = Vec::new();

        // 1. Initial calculation for all assets
        for (index, item) in self.calculators.iter().enumerate() {
            match item.calculate_zakat(config) {
                Ok(detail) => results.push(PortfolioItemResult::Success(detail)),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = item.get_label() {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    err = err.with_source(source.clone());
                    results.push(PortfolioItemResult::Failure {
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
        let mut results = Vec::new();

        for (index, item) in self.calculators.iter().enumerate() {
            match item.calculate_zakat_async(config).await {
                Ok(detail) => results.push(PortfolioItemResult::Success(detail)),
                Err(e) => {
                    let mut err = e;
                    let source = if let Some(lbl) = item.get_label() {
                        lbl
                    } else {
                        format!("Item {}", index + 1)
                    };
                    err = err.with_source(source.clone());
                    results.push(PortfolioItemResult::Failure {
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
fn aggregate_and_summarize(mut results: Vec<PortfolioItemResult>, config: &crate::config::ZakatConfig) -> PortfolioResult {
    // 2. Aggregation Logic (Dam' al-Amwal)
    // Filter monetary assets (Gold, Silver, Cash, Business, Investments) from SUCCESSFUL results
    let mut monetary_net_assets = Decimal::ZERO;
    let mut monetary_indices = Vec::new();

    for (i, result) in results.iter().enumerate() {
        if let PortfolioItemResult::Success(detail) = result
            && detail.wealth_type.is_monetary()
        {
            monetary_net_assets += detail.net_assets;
            monetary_indices.push(i);
        }
    }
    
    // Check against the global monetary Nisab
    let global_nisab = config.get_monetary_nisab_threshold();
    
    if monetary_net_assets >= global_nisab && monetary_net_assets > Decimal::ZERO {
        let standard_rate = rust_decimal_macros::dec!(0.025);

        for i in monetary_indices {
            // We need to mutate the result.
            if let Some(PortfolioItemResult::Success(detail)) = results.get_mut(i)
                && !detail.is_payable
            {
                detail.is_payable = true;
                detail.status_reason = Some("Payable via Aggregation (Dam' al-Amwal)".to_string());
                
                // Recalculate zakat due
                if detail.net_assets > Decimal::ZERO {
                    detail.zakat_due = detail.net_assets * standard_rate;
                }
                
                // Add trace step explaining aggregation
                detail.calculation_trace.push(crate::types::CalculationStep::info(
                    "Aggregated Monetary Wealth > Nisab -> Payable (Dam' al-Amwal)"
                ));
                detail.calculation_trace.push(crate::types::CalculationStep::result(
                    "Recalculated Zakat Due", detail.zakat_due
                ));
            }
        }
    }

    // 3. Final Summation (only successes)
    let mut total_assets = Decimal::ZERO;
    let mut total_zakat_due = Decimal::ZERO;
    let items_attempted = results.len();
    let items_failed = results.iter().filter(|r| matches!(r, PortfolioItemResult::Failure { .. })).count();

    for result in &results {
        if let PortfolioItemResult::Success(detail) = result {
            total_assets += detail.total_assets;
            total_zakat_due += detail.zakat_due;
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


