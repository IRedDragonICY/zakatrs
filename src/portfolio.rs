use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::traits::CalculateZakat;
use crate::types::{ZakatDetails, ZakatError};

/// Result of a portfolio calculation.
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioResult {
    pub details: Vec<ZakatDetails>,
    pub total_assets: Decimal,
    pub total_zakat_due: Decimal,
}

pub struct ZakatPortfolio {
    calculators: Vec<Box<dyn CalculateZakat + Send + Sync>>,
    // We might want to store specific debts for the portfolio if global? 
    // Or just rely on individual calculators having debts passed in?
    // The requirement says: "ZakatPortfolio::new().add_income(...).build()"
    // And "calculate_total(&self, config) -> PortfolioResult"
    // But `CalculateZakat::calculate_zakat` takes `debts`.
    // If we want to support debts per item, we should probably wrap them with their debts?
    // Or allow the user to pass debts during `add` phase.
    // For now, let's assume `add_*(..., debt)` or adhering to the simple signature.
    // Let's store closures or boxed structs that ALREADY have everything needed?
    // But commonly, `CalculateZakat` implementation needs `debts` passed at calc time.
    // Let's wrap the logic.
}

// Check design: trait `calculate_zakat(&self, debts: Option<Decimal>)`.
// If we want to automate this, we need to know what debts to pass.
// Maybe we can store `(Box<dyn CalculateZakat>, Option<Decimal>)`.

impl ZakatPortfolio {
    pub fn new() -> Self {
        Self {
            calculators: Vec::new(),
        }
    }

    pub fn add_calculator<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T) -> Self {
         // This assumes no extra debt passed at calc time, or debt is handled inside calculator if possible?
         // Actually, our calculators don't store debt (except Business which stores liabilities).
         // Most rely on `calculate_zakat(debts)`.
         // If we use this generic list, we can't easily pass different debts to different items unless we wrap them.
         // Let's create a wrapper that holds the calculator AND the specific debt to be deducted.
         self.calculators.push(Box::new(PortfolioItemWrapper { calculator, debt: None }));
         self
    }
    
    pub fn add_calculator_with_debt<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T, debt: Decimal) -> Self {
         self.calculators.push(Box::new(PortfolioItemWrapper { calculator, debt: Some(debt) }));
         self
    }

    // Specific helpers to make it "Developer Friendly" as requested
    // "add_income(...)", "add_gold(...)"
    // These require constructing the structs.
    
    pub fn calculate_total(&self) -> Result<PortfolioResult, ZakatError> {
        let mut details = Vec::new();
        let mut total_assets = Decimal::ZERO;
        let mut total_zakat_due = Decimal::ZERO;

        for item in &self.calculators {
            let detail = item.calculate_zakat(None)?; // Wrapper handles the debt passing
            total_assets += detail.total_assets;
            total_zakat_due += detail.zakat_due;
            details.push(detail);
        }

        Ok(PortfolioResult {
            details,
            total_assets,
            total_zakat_due,
        })
    }
}

// Wrapper to hold debt context for the generic list
struct PortfolioItemWrapper<T: CalculateZakat> {
    calculator: T,
    debt: Option<Decimal>,
}

impl<T: CalculateZakat> CalculateZakat for PortfolioItemWrapper<T> {
    fn calculate_zakat(&self, _ignored_debts: Option<Decimal>) -> Result<ZakatDetails, ZakatError> {
        // We use the stored debt. We ignore the arg passed to THIS wrapper usually,
        // or we could combine them. For Portfolio iteration, we pass None typically.
        self.calculator.calculate_zakat(self.debt)
    }
}
