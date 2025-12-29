use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::traits::CalculateZakat;
use crate::types::ZakatDetails;

/// Result of a portfolio calculation, including any partial failures.
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioReport {
    pub details: Vec<ZakatDetails>,
    pub errors: Vec<String>,
    pub total_assets: Decimal,
    pub total_zakat_due: Decimal,
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
    pub fn calculate_total(&self, config: &crate::config::ZakatConfig) -> PortfolioReport {
        let mut details = Vec::new();
        let mut errors = Vec::new();

        // 1. Initial calculation for all assets
        for (index, item) in self.calculators.iter().enumerate() {
            match item.calculate_zakat(config) {
                Ok(detail) => details.push(detail),
                Err(e) => {
                    let mut err = e;
                    if let Some(lbl) = item.get_label() {
                        err = err.with_source(lbl);
                    } else {
                        err = err.with_source(format!("Item {}", index + 1));
                    }
                    errors.push(err.to_string())
                },
            }
        }

        // 2. Aggregation Logic (Dam' al-Amwal)
        // Filter monetary assets (Gold, Silver, Cash, Business, Investments)
        let mut monetary_net_assets = Decimal::ZERO;
        let mut monetary_indices = Vec::new();

        for (i, detail) in details.iter().enumerate() {
            if detail.wealth_type.is_monetary() {
                monetary_net_assets += detail.net_assets;
                monetary_indices.push(i);
            }
        }

        // Check against the global monetary Nisab (usually 85g Gold value)
        let global_nisab = config.get_monetary_nisab_threshold();

        // If the total aggregated monetary wealth meets the Nisab, they are all payable
        if monetary_net_assets >= global_nisab && monetary_net_assets > Decimal::ZERO {
            // Rate is typically 2.5% (0.025) for these assets. 
            // We can assume standard rate or re-use the rate implied by zakat_due/net_assets, 
            // but hardcoding 0.025 for monetary assets is safe in this context as per Fiqh.
            let standard_rate = rust_decimal_macros::dec!(0.025);

            for i in monetary_indices {
                let detail = &mut details[i];
                
                // If it wasn't already payable (or even if it was, we confirm it here),
                // we mark it payable and ensure Zakat is calculated on its net assets.
                if !detail.is_payable {
                     detail.is_payable = true;
                     detail.status_reason = Some("Payable via Aggregation (Dam' al-Amwal)".to_string());
                     
                     // Recalculate zakat due for this specific component
                     if detail.net_assets > Decimal::ZERO {
                         detail.zakat_due = detail.net_assets * standard_rate;
                     }
                }
            }
        }

        // 3. Final Summation
        let mut total_assets = Decimal::ZERO;
        let mut total_zakat_due = Decimal::ZERO;

        for detail in &details {
            total_assets += detail.total_assets;
            total_zakat_due += detail.zakat_due;
        }

        PortfolioReport {
            details,
            errors,
            total_assets,
            total_zakat_due,
        }
    }
}


