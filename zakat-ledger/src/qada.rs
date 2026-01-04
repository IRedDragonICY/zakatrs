use rust_decimal::Decimal;
use chrono::NaiveDate;
use zakat_core::types::ZakatError;
use zakat_providers::HistoricalPriceProvider;
use serde::{Serialize, Deserialize};

/// Result of a Zakat calculation for a specific historical lunar year.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QadaYearResult {
    /// Approximate Gregorian date representing the end of the historical Hawl.
    pub date: NaiveDate,
    /// Sequential index of the year being reconstructed.
    pub year_index: usize,
    /// Estimated wealth held by the user at that specific point in time.
    pub wealth_amount: Decimal,
    /// Historical gold price per gram in the local currency.
    pub gold_price: Decimal,
    /// Historical silver price per gram in the local currency.
    pub silver_price: Decimal,
    /// Calculated Nisab threshold based on the selected standard and historical prices.
    pub nisab_threshold: Decimal,
    /// Indicates if the user's wealth exceeded the Nisab threshold for this year.
    pub is_payable: bool,
    /// The final Zakat amount due for this specific year (usually 2.5% of total wealth).
    pub zakat_due: Decimal,
    /// Contextual notes regarding the calculation (e.g., data source fallbacks).
    pub notes: Option<String>,
}

/// Final report aggregating multiple years of missed Zakat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QadaReport {
    /// Grand total of Zakat payable across the entire history provided.
    pub total_due: Decimal,
    /// The number of years successfully processed in this report.
    pub years_count: usize,
    /// A chronological breakdown of each year's calculation.
    pub year_details: Vec<QadaYearResult>,
}

/// Reconstructs historical Zakat liabilities for missed years (Qada).
/// 
/// This engine is designed to handle "Joining Wealth" across several years, 
/// using historical price data to determine contextual Nisab thresholds.
pub struct QadaCalculator<P> {
    price_provider: P,
}

impl<P: HistoricalPriceProvider> QadaCalculator<P> {
    /// Creates a new QadaCalculator with a historical price provider.
    pub fn new(price_provider: P) -> Self {
        Self { price_provider }
    }

    /// Calculates Qada Zakat for a series of wealth snapshots.
    ///
    /// # Arguments
    /// * `wealth_history`: A vector of (Date, WealthAmount) tuples or structs.
    ///                   Ideally, provide one entry per lunar year.
    ///                   If dates are irregular, this method calculates strictly based on the provided snapshots.
    /// * `madhab_config`: Configuration for Nisab standard (Hanafi/Silver vs Shafi/Gold).
    ///
    /// # Returns
    /// A `QadaReport` containing total dues and year-by-year breakdown.
    pub async fn calculate_snapshots(
        &self, 
        wealth_history: &[(NaiveDate, Decimal)], 
        nisab_standard: zakat_core::madhab::NisabStandard
    ) -> Result<QadaReport, ZakatError> {
        let mut total_due = Decimal::ZERO;
        let mut details = Vec::new();

        for (idx, (date, wealth)) in wealth_history.iter().enumerate() {
            // 1. Fetch historical prices
            // Note: If exact date prices missing, provider handles fallback usually.
            let prices = self.price_provider.get_prices_on(*date).await?;
            
            // 2. Calculate Nisab
            use zakat_core::madhab::NisabStandard;
            use rust_decimal_macros::dec;
            
            let gold_nisab = prices.gold_per_gram * dec!(85.0);
            let silver_nisab = prices.silver_per_gram * dec!(595.0);
            
            let threshold = match nisab_standard {
                NisabStandard::Gold => gold_nisab,
                NisabStandard::Silver => silver_nisab,
                NisabStandard::LowerOfTwo => {
                     // Check if silver price is zero (data missing), fallback to gold?
                     if prices.silver_per_gram.is_zero() {
                         gold_nisab
                     } else {
                         gold_nisab.min(silver_nisab)
                     }
                }
            };
            
            // 3. Determine Payability
            let is_payable = *wealth >= threshold;
            let zakat_due = if is_payable {
                *wealth * dec!(0.025)
            } else {
                Decimal::ZERO
            };

            if is_payable {
                total_due += zakat_due;
            }

            details.push(QadaYearResult {
                date: *date,
                year_index: idx + 1,
                wealth_amount: *wealth,
                gold_price: prices.gold_per_gram,
                silver_price: prices.silver_per_gram,
                nisab_threshold: threshold,
                is_payable,
                zakat_due,
                notes: if prices.silver_per_gram.is_zero() && nisab_standard != NisabStandard::Gold {
                    Some("Silver price missing, used Gold/Fallback.".to_string()) 
                } else { 
                    None 
                },
            });
        }

        Ok(QadaReport {
            total_due,
            years_count: details.len(),
            year_details: details,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use zakat_providers::{StaticHistoricalPriceProvider, Prices};

    #[tokio::test]
    async fn test_qada_calculation() {
        // Setup mock prices:
        // Year 1 (2010): Gold $40/g, Silver $0.6/g (Nisab Gold (~$3400), Silver (~$357))
        // Year 2 (2011): Gold $50/g, Silver $1.0/g (Nisab Gold (~$4250), Silver (~$595))
        let p1 = Prices { gold_per_gram: dec!(40.0), silver_per_gram: dec!(0.6) };
        let p2 = Prices { gold_per_gram: dec!(50.0), silver_per_gram: dec!(1.0) };
        
        let provider = StaticHistoricalPriceProvider::new()
            .with_price(NaiveDate::from_ymd_opt(2010, 8, 11).unwrap(), p1)
            .with_price(NaiveDate::from_ymd_opt(2011, 8, 1).unwrap(), p2);

        let calculator = QadaCalculator::new(provider);

        let history = vec![
            (NaiveDate::from_ymd_opt(2010, 8, 11).unwrap(), dec!(5000.0)), // Above Silver & Gold Nisab -> Payable
            (NaiveDate::from_ymd_opt(2011, 8, 1).unwrap(), dec!(4000.0)),  // Above Silver ($595), Below Gold ($4250)? 4000 < 4250.
        ];

        // Case 1: Silver Standard (LowerOfTwo in this context) -> Both payable
        let report_silver = calculator.calculate_snapshots(&history, zakat_core::madhab::NisabStandard::Silver).await.unwrap();
        assert_eq!(report_silver.year_details[0].is_payable, true);
        assert_eq!(report_silver.year_details[1].is_payable, true); // 4000 > 595
        assert_eq!(report_silver.total_due, dec!(125.0) + dec!(100.0)); // 2.5% of 5000 (125) + 2.5% of 4000 (100)

        // Case 2: Gold Standard -> Year 2 might be exempt
        let report_gold = calculator.calculate_snapshots(&history, zakat_core::madhab::NisabStandard::Gold).await.unwrap();
        assert_eq!(report_gold.year_details[0].is_payable, true); // 5000 > 3400 (85*40)
        assert_eq!(report_gold.year_details[1].is_payable, false); // 4000 < 4250 (85*50)
        assert_eq!(report_gold.total_due, dec!(125.0));
    }
}
