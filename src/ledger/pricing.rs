use rust_decimal::Decimal;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

use crate::types::{ZakatError, ErrorDetails};

pub trait HistoricalPriceProvider {
    fn get_nisab_threshold(&self, date: NaiveDate) -> Result<Decimal, ZakatError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryPriceHistory {
    prices: BTreeMap<NaiveDate, Decimal>,
}

impl Default for InMemoryPriceHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryPriceHistory {
    pub fn new() -> Self {
        Self {
            prices: BTreeMap::new(),
        }
    }

    pub fn add_price(&mut self, date: NaiveDate, price: Decimal) {
        self.prices.insert(date, price);
    }
}

impl HistoricalPriceProvider for InMemoryPriceHistory {
    fn get_nisab_threshold(&self, date: NaiveDate) -> Result<Decimal, ZakatError> {
         // Return the price for the specific date if it exists. 
         // For a clearer simulation, we might want to look for the *most recent* price 
         // before or on that date.
         
         self.prices.range(..=date).next_back().map(|(_, &price)| price)
            .ok_or_else(|| ZakatError::ConfigurationError(Box::new(ErrorDetails { 
                reason_key: "error-nisab-price-missing".to_string(),
                args: Some(std::collections::HashMap::from([("date".to_string(), date.to_string())])),
                source_label: Some("HistoricalPriceProvider".to_string()),
                asset_id: None
            })))
    }
}
