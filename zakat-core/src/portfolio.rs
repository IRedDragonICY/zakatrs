//! # Fiqh Compliance: Portfolio Aggregation
//!
//! ## Principle: Dam' al-Amwal (Joining Wealth)
//! - Implements the **Hanafi** and Majority view that Gold, Silver, Cash, and Trade Goods are of a single genus (*Thamaniyyah*) and must be combined to reach the Nisab.
//! - **Benefit**: This ensures the poor receive their due from wealth that would otherwise be exempt if split (*Anfa' lil-fuqara*).

use rust_decimal::Decimal;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::traits::CalculateZakat;
#[cfg(feature = "async")]
use crate::traits::AsyncCalculateZakat;
use crate::types::{ZakatDetails, ZakatError, ErrorDetails, InvalidInputDetails};
use crate::assets::PortfolioItem;
use tracing::{instrument, info, warn};

// =============================================================================
// Portfolio Snapshot (Feature 2: Audit Logs)
// =============================================================================

/// A point-in-time snapshot of a portfolio calculation for audit purposes.
///
/// This struct captures the exact state of a calculation including:
/// - The configuration used (with prices at that moment).
/// - All input assets.
/// - The calculation result.
/// - User-defined metadata (notes, tax year, etc.).
///
/// # Use Cases
/// - Tax documentation and reporting.
/// - Audit trails for compliance.
/// - Historical record keeping.
/// - Comparing calculations over time.
///
/// # Example
/// ```rust,ignore
/// let result = portfolio.calculate_total(&config);
/// let snapshot = portfolio.snapshot(&config, &result)
///     .with_metadata("tax_year", "2025")
///     .with_metadata("prepared_by", "Ahmad");
/// 
/// // Serialize for storage
/// let json = serde_json::to_string_pretty(&snapshot)?;
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSnapshot {
    /// Unique identifier for this snapshot.
    pub id: Uuid,
    /// UTC timestamp when the snapshot was created.
    pub timestamp: DateTime<Utc>,
    /// The configuration used for this calculation (including prices).
    pub config_snapshot: crate::config::ZakatConfig,
    /// The input assets at the time of calculation.
    pub inputs: Vec<PortfolioItem>,
    /// The calculation result.
    pub result: PortfolioResult,
    /// User-defined metadata (e.g., notes, tax year, preparer).
    pub metadata: HashMap<String, String>,
    /// Version of the snapshot format (for future compatibility).
    pub version: String,
}

impl PortfolioSnapshot {
    /// Creates a new snapshot with the current timestamp.
    pub fn new(
        config: &crate::config::ZakatConfig,
        inputs: Vec<PortfolioItem>,
        result: PortfolioResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            config_snapshot: config.clone(),
            inputs,
            result,
            metadata: HashMap::new(),
            version: "1.0.0".to_string(),
        }
    }

    /// Adds a metadata entry to the snapshot.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Sets multiple metadata entries at once.
    pub fn with_all_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Returns the total Zakat due from this snapshot.
    pub fn total_zakat_due(&self) -> Decimal {
        self.result.total_zakat_due
    }

    /// Returns the total assets from this snapshot.
    pub fn total_assets(&self) -> Decimal {
        self.result.total_assets
    }

    /// Returns the snapshot as a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Creates a snapshot from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Returns a summary string for display.
    pub fn summary(&self) -> String {
        format!(
            "Portfolio Snapshot ({})\n\
             Date: {}\n\
             Assets: {} items\n\
             Total Zakat Due: {:.2}\n\
             Status: {:?}",
            self.id,
            self.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            self.inputs.len(),
            self.result.total_zakat_due,
            self.result.status
        )
    }
}

// =============================================================================
// Portfolio Item Result
// =============================================================================

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
    /// Successfully calculated details (v1.2+).
    pub successes: Vec<ZakatDetails>,
    /// Failed calculations (v1.2+).
    pub failures: Vec<PortfolioItemResult>,
    pub total_assets: Decimal,
    pub total_zakat_due: Decimal,
    pub items_attempted: usize,
    pub items_failed: usize,
}

impl PortfolioResult {
    /// Returns a list of failed calculations.
    pub fn failures(&self) -> &Vec<PortfolioItemResult> {
        &self.failures
    }

    /// Returns a list of successful calculation details.
    pub fn successes(&self) -> &Vec<ZakatDetails> {
        &self.successes
    }

    /// Reconstructs the legacy results list for backward compatibility.
    pub fn results(&self) -> Vec<PortfolioItemResult> {
        let mut list = Vec::with_capacity(self.successes.len() + self.failures.len());
        for s in &self.successes {
            list.push(PortfolioItemResult::Success {
                asset_id: s.asset_id.unwrap_or_else(Uuid::nil),
                details: s.clone()
            });
        }
        list.extend(self.failures.clone());
        list
    }

    /// Returns true if there were no failures.
    pub fn is_clean(&self) -> bool {
        self.status == PortfolioStatus::Complete
    }
    
    /// Returns the result if Complete, otherwise returns an error describing the failure(s).
    pub fn expect_complete(self) -> Result<Self, ZakatError> {
        match self.status {
            PortfolioStatus::Complete => Ok(self),
            PortfolioStatus::Partial => Err(ZakatError::CalculationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::CalculationError,
                reason_key: "error-portfolio-incomplete".to_string(),
                args: Some(std::collections::HashMap::from([
                    ("failed".to_string(), self.items_failed.to_string()),
                    ("attempted".to_string(), self.items_attempted.to_string())
                ])), 
                source_label: Some("Portfolio".to_string()),
                suggestion: Some("Check individual asset errors and retry.".to_string()),
                ..Default::default()
            }))),
            PortfolioStatus::Failed => Err(ZakatError::CalculationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::CalculationError,
                reason_key: "error-portfolio-failed".to_string(), 
                source_label: Some("Portfolio".to_string()),
                suggestion: Some("All asset calculations failed. Check configuration.".to_string()),
                ..Default::default()
            }))),
        }
    }
}

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

    #[allow(clippy::should_implement_trait)]
    pub fn add<T: Into<PortfolioItem>>(mut self, item: T) -> Self {
         self.items.push(item.into());
         self
    }

    /// Convenience method to add a cash asset.
    pub fn add_cash(self, amount: impl crate::inputs::IntoZakatDecimal, label: &str) -> Self {
        use crate::maal::business::BusinessZakat;
        self.add(BusinessZakat::cash_only(amount).label(label))
    }

    /// Convenience method to add a gold asset.
    pub fn add_gold(self, grams: impl crate::inputs::IntoZakatDecimal, purity_24k: impl crate::inputs::IntoZakatDecimal) -> Self {
        use crate::maal::precious_metals::PreciousMetals;
        self.add(PreciousMetals::gold(grams).purity(purity_24k))
    }

    /// Convenience method to add a silver asset.
    pub fn add_silver(self, grams: impl crate::inputs::IntoZakatDecimal) -> Self {
        use crate::maal::precious_metals::PreciousMetals;
        self.add(PreciousMetals::silver(grams))
    }

    /// Convenience method to add a business asset.
    pub fn add_business(self, cash: impl crate::inputs::IntoZakatDecimal, inventory: impl crate::inputs::IntoZakatDecimal, receivables: impl crate::inputs::IntoZakatDecimal, debts: impl crate::inputs::IntoZakatDecimal) -> Self {
        use crate::maal::business::BusinessZakat;
        self.add(BusinessZakat::new()
            .cash(cash)
            .inventory(inventory)
            .receivables(receivables)
            .add_liability("Short-term Debt", debts))
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
            Err(ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                code: crate::types::ZakatErrorCode::AssetNotFound,
                field: "asset_id".to_string(),
                value: id.to_string(),
                reason_key: "error-asset-not-found".to_string(),
                suggestion: Some("Asset with this ID does not exist in the portfolio.".to_string()),
                ..Default::default()
            })))
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

    /// Gets a reference to an asset by its human-readable label.
    ///
    /// Returns the first asset with a matching label, or `None` if no match is found.
    /// ```rust,ignore
    /// portfolio.get_by_label("Gold Savings").map(|item| { /* use item */ });
    /// ```
    pub fn get_by_label(&self, label: &str) -> Option<&PortfolioItem> {
        self.items.iter().find(|item| {
            CalculateZakat::get_label(*item).as_deref() == Some(label)
        })
    }

    /// Gets a mutable reference to an asset by its human-readable label.
    ///
    /// Returns the first asset with a matching label, or `None` if no match is found.
    /// ```rust,ignore
    /// portfolio.get_by_label_mut("Gold Savings").map(|item| { /* modify item */ });
    /// ```
    pub fn get_by_label_mut(&mut self, label: &str) -> Option<&mut PortfolioItem> {
        self.items.iter_mut().find(|item| {
            CalculateZakat::get_label(&**item).as_deref() == Some(label)
        })
    }

    /// Removes an asset by its human-readable label.
    ///
    /// Returns the removed item if found, or `None` if no match exists.
    /// ```rust,ignore
    /// if let Some(removed) = portfolio.remove_by_label("Gold Savings") {
    ///     println!("Removed: {:?}", removed);
    /// }
    /// ```
    pub fn remove_by_label(&mut self, label: &str) -> Option<PortfolioItem> {
        if let Some(pos) = self.items.iter().position(|item| {
            CalculateZakat::get_label(item).as_deref() == Some(label)
        }) {
            Some(self.items.remove(pos))
        } else {
            None
        }
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
                successes: Vec::new(),
                failures: vec![PortfolioItemResult::Failure {
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
                successes: Vec::new(),
                failures: vec![PortfolioItemResult::Failure {
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

        let mut new_results = Vec::with_capacity(previous_result.items_attempted);
        
        // 1. Keep existing successes
        for success in &previous_result.successes {
            new_results.push(PortfolioItemResult::Success { 
                asset_id: success.asset_id.unwrap_or_else(Uuid::nil), 
                details: success.clone() 
            });
        }
        
        // 2. Retry failures
        for failure in &previous_result.failures {
            if let PortfolioItemResult::Failure { asset_id, source, .. } = failure {
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
                     new_results.push(failure.clone());
                 }
            }
        }
        
        aggregate_and_summarize(new_results, config)
    }

    /// Creates a snapshot of the current portfolio calculation for audit purposes.
    ///
    /// A snapshot captures the exact state of a calculation including:
    /// - The configuration used (with prices at that moment).
    /// - All input assets.
    /// - The calculation result.
    ///
    /// # Example
    /// ```rust,ignore
    /// let result = portfolio.calculate_total(&config);
    /// let snapshot = portfolio.snapshot(&config, &result)
    ///     .with_metadata("tax_year", "2025")
    ///     .with_metadata("notes", "End of Ramadan calculation");
    /// ```
    pub fn snapshot(&self, config: &crate::config::ZakatConfig, result: &PortfolioResult) -> PortfolioSnapshot {
        PortfolioSnapshot::new(config, self.items.clone(), result.clone())
    }

    /// Generates a comprehensive, human-readable report of the entire portfolio.
    /// 
    /// This method aggregates individual asset explanations, calculation steps, 
    /// and any warnings into a single structured string. It is ideal for 
    /// CLI output or providing detailed feedback to end-users.
    pub fn explain(&self, config: &crate::config::ZakatConfig, translator: &impl crate::traits::Translator) -> String {
        let result = self.calculate_total(config);
        let mut output = String::new();
        
        use std::fmt::Write;
        
        writeln!(&mut output, "{}", translator.translate("report-header", None)).ok();
        writeln!(&mut output, "{}: {}", translator.translate("report-total-zakat", None), result.total_zakat_due).ok();
        writeln!(&mut output, "{}: {}", translator.translate("report-total-assets", None), result.total_assets).ok();
        writeln!(&mut output, "==============================").ok();
        
        output.push_str(&format!("\n{}\n", translator.translate("report-breakdown-title", None)));
        
        // Mix successes and failures by reconstructing order or just listing successes then failures
        // For CLI report, order matters (input order).
        // Since we split them, we lost input order unless we sort by something?
        // But ZakatDetails has label.
        // Ideally we iterate `items` and find result?
        // Or we iterate `self.items` and lookup in result.
        
        for (idx, item) in self.items.iter().enumerate() {
            let item_id = crate::traits::CalculateZakat::get_id(item);
            let label = crate::traits::CalculateZakat::get_label(item).unwrap_or_else(|| format!("Asset #{}", idx + 1));
            
            // Find in success
            if let Some(details) = result.successes.iter().find(|s| s.asset_id == Some(item_id)) {
                writeln!(&mut output, "\n[{}]: {}", label, details.status_reason.as_deref().unwrap_or("Unknown")).ok();
                writeln!(&mut output, "  - {}: {}", translator.translate("report-value", None), details.total_assets).ok(); // Simplified lookup
                writeln!(&mut output, "  - {}: {}", translator.translate("report-zakat-due", None), details.zakat_due).ok();
                
                if !details.structured_warnings.is_empty() {
                    writeln!(&mut output, "  - Warnings:").ok();
                    for w in &details.structured_warnings {
                        writeln!(&mut output, "    * {}", w.message).ok();
                    }
                }
            } else if let Some(fail) = result.failures.iter().find(|f| matches!(f, PortfolioItemResult::Failure{asset_id, ..} if *asset_id == item_id)) {
                 if let PortfolioItemResult::Failure { error, .. } = fail {
                    writeln!(&mut output, "\n[{}]: {}", label, translator.translate("report-failed", None)).ok();
                    writeln!(&mut output, "  - Error: {}", error).ok();
                 }
            }
        }
        
        output
    }

    /// Performs a non-destructive "What-If" simulation on the portfolio.
    ///
    /// This method clones the current portfolio and applies the provided closure 
    /// to the clone, allowing for temporary modifications (e.g., adding a hypothetical 
    /// asset or changing a price) without affecting the actual portfolio state.
    ///
    /// # Arguments
    /// * `func` - A closure that modifies the cloned portfolio before calculation.
    /// * `config` - The configuration and prices to use for the simulation.
    ///
    /// # Returns
    /// A `PortfolioResult` representing the outcome of the simulated state.
    ///
    /// # Example
    /// ```rust,ignore
    /// let result = portfolio.simulate_change(|p| {
    ///     p.push(BusinessZakat::new().cash(5000));
    /// }, &config);
    /// ```
    pub fn simulate_change<F>(&self, func: F, config: &crate::config::ZakatConfig) -> PortfolioResult 
    where
        F: FnOnce(&mut Self)
    {
        let mut sim = self.clone();
        func(&mut sim);
        sim.calculate_total(config)
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
    
    /// Calculates Zakat asynchronously for all assets in the portfolio.
    #[instrument(skip(self, config), fields(items_count = self.items.len()))]
    pub async fn calculate_total_async(&self, config: &crate::config::ZakatConfig) -> PortfolioResult {
        info!("Starting async portfolio calculation");
        // Fail Fast: Validate config before processing any items
        if let Err(e) = config.validate() {
            return PortfolioResult {
                status: PortfolioStatus::Failed,
                successes: Vec::new(),
                failures: vec![PortfolioItemResult::Failure {
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
                    details.calculation_breakdown.push(crate::types::CalculationStep::info(
                        "info-aggregation-payable",
                        "Aggregated Monetary Wealth > Nisab -> Payable (Dam' al-Amwal)"
                    ));
                    details.calculation_breakdown.push(crate::types::CalculationStep::result(
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

    let mut successes = Vec::with_capacity(results.len());
    let mut failures = Vec::with_capacity(results.len());

    for result in results {
        match result {
            PortfolioItemResult::Success { details, .. } => successes.push(details),
            r @ PortfolioItemResult::Failure { .. } => failures.push(r),
        }
    }

    PortfolioResult {
        status,
        successes,
        failures,
        total_assets,
        total_zakat_due,
        items_attempted,
        items_failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZakatConfig;
    use rust_decimal_macros::dec;
    use crate::maal::business::BusinessZakat;

    struct MockTranslator;
    impl crate::traits::Translator for MockTranslator {
         fn translate(&self, key: &str, _args: Option<&std::collections::HashMap<String, String>>) -> String {
             // Return meaningful strings for assertions
             if key == "report-header" { return "=== Zakat Portfolio Report ===".to_string(); }
             if key == "report-total-zakat" { return "Total Zakat Due".to_string(); }
             if key == "report-total-assets" { return "Total Assets".to_string(); }
             if key == "report-breakdown-title" { return "Asset Breakdown".to_string(); }
             key.to_string()
         }
    }

    #[test]
    fn test_portfolio_explain_aggregation() {
        let mut portfolio = ZakatPortfolio::new();
        let config = ZakatConfig::test_default().with_gold_price(dec!(100));
        
        // 1. Business asset worth 10000 (Above Nisab 8500)
        let biz = BusinessZakat::new().cash(10000).label("Shop").hawl(true);
        portfolio = portfolio.add(biz);
        
        let translator = MockTranslator;
        let explanation = portfolio.explain(&config, &translator);
        
        assert!(explanation.contains("=== Zakat Portfolio Report ==="));
        assert!(explanation.contains("Total Zakat Due"));
        assert!(explanation.contains("250"));
        assert!(explanation.contains("[Shop]"));
    }

    #[test]
    fn test_simulator_non_destructive() {
        let mut portfolio = ZakatPortfolio::new();
        let config = ZakatConfig::test_default();
        
        portfolio = portfolio.add(BusinessZakat::new().cash(1000));
        let original_count = portfolio.items.len();
        
        let sim_res = portfolio.simulate_change(|p| {
            p.push(BusinessZakat::new().cash(5000));
        }, &config);
        
        assert_eq!(portfolio.items.len(), original_count, "Original portfolio should not be modified");
        assert_eq!(sim_res.items_attempted, 2, "Simulated result should have 2 items");
    }
}
