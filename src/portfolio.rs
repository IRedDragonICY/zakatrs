use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::traits::CalculateZakat;
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

/// Result of a portfolio calculation, including successes and partial failures.
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioResult {
    pub results: Vec<PortfolioItemResult>,
    pub total_assets: Decimal,
    pub total_zakat_due: Decimal,
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
        !self.results.iter().any(|r| matches!(r, PortfolioItemResult::Failure { .. }))
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

        // 2. Aggregation Logic (Dam' al-Amwal)
        // Filter monetary assets (Gold, Silver, Cash, Business, Investments) from SUCCESSFUL results
        let mut monetary_net_assets = Decimal::ZERO;
        let mut monetary_indices = Vec::new();

        for (i, result) in results.iter().enumerate() {
            if let PortfolioItemResult::Success(detail) = result {
                 if detail.wealth_type.is_monetary() {
                    monetary_net_assets += detail.net_assets;
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
                if let Some(PortfolioItemResult::Success(detail)) = results.get_mut(i) {
                    if !detail.is_payable {
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
        }

        // 3. Final Summation (only successes)
        let mut total_assets = Decimal::ZERO;
        let mut total_zakat_due = Decimal::ZERO;

        for result in &results {
            if let PortfolioItemResult::Success(detail) = result {
                total_assets += detail.total_assets;
                total_zakat_due += detail.zakat_due;
            }
        }

        PortfolioResult {
            results,
            total_assets,
            total_zakat_due,
        }
    }
}


