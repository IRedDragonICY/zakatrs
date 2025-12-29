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
         self.calculators.push(Box::new(calculator));
         self
    }

    // Helper methods for specific calculator types can be added here.
    
    pub fn calculate_total(&self) -> Result<PortfolioResult, ZakatError> {
        let mut details = Vec::new();
        let mut total_assets = Decimal::ZERO;
        let mut total_zakat_due = Decimal::ZERO;

        for item in &self.calculators {
            let detail = item.calculate_zakat()?; 
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


