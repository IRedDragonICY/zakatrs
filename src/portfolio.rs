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
}

// Wrapper handles storing the item-specific debt context if provided.

impl ZakatPortfolio {
    pub fn new() -> Self {
        Self {
            calculators: Vec::new(),
        }
    }

    pub fn add_calculator<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T) -> Self {
         // Wraps the calculator with no specific debt (None).
         // The debt handled here is specific to this portfolio item.
         self.calculators.push(Box::new(PortfolioItemWrapper { calculator, debt: None }));
         self
    }
    
    pub fn add_calculator_with_debt<T: CalculateZakat + Send + Sync + 'static>(mut self, calculator: T, debt: Decimal) -> Self {
         self.calculators.push(Box::new(PortfolioItemWrapper { calculator, debt: Some(debt) }));
         self
    }

    // Helper methods for specific calculator types can be added here.
    
    pub fn calculate_total(&self) -> Result<PortfolioResult, ZakatError> {
        let mut details = Vec::new();
        let mut total_assets = Decimal::ZERO;
        let mut total_zakat_due = Decimal::ZERO;

        for item in &self.calculators {
            let detail = item.calculate_zakat(None, true)?; // Wrapper handles the debt passing, Hawl assumed true for portfolio total
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
    fn calculate_zakat(&self, _ignored_debts: Option<Decimal>, hawl_satisfied: bool) -> Result<ZakatDetails, ZakatError> {
        // We use the stored debt. We ignore the arg passed to THIS wrapper usually,
        // or we could combine them. For Portfolio iteration, we pass None typically.
        self.calculator.calculate_zakat(self.debt, hawl_satisfied)
    }
}
