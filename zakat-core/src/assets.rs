//! Asset Wrapper Types
//!
//! This module defines the `PortfolioItem` enum which wraps all zakatable asset types
//! for uniform handling in portfolios.

use serde::{Serialize, Deserialize};
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::{CalculateZakat, ZakatConfigArgument};

use crate::maal::business::BusinessZakat;
use crate::maal::income::IncomeZakatCalculator;
use crate::maal::livestock::LivestockAssets;
use crate::maal::agriculture::AgricultureAssets;
use crate::maal::investments::InvestmentAssets;
use crate::maal::mining::MiningAssets;
use crate::maal::precious_metals::PreciousMetals;
use crate::fitrah::FitrahCalculator;
use rust_decimal::Decimal;
use uuid::Uuid;

/// Generic asset type for user-defined assets.
/// Allows users to create custom zakatable assets with custom rates and thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(rename_all = "camelCase")]
pub struct CustomAsset {
    /// Unique identifier for this asset.
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    /// Human-readable label for this asset.
    pub label: String,
    /// Total value of the asset.
    #[typeshare(serialized_as = "string")]
    pub value: Decimal,
    /// Zakat rate to apply (e.g., 0.025 for 2.5%).
    #[typeshare(serialized_as = "string")]
    pub rate: Decimal,
    /// Minimum threshold for Zakat obligation.
    #[typeshare(serialized_as = "string")]
    pub nisab_threshold: Decimal,
    /// Whether the Hawl (one lunar year) has been satisfied.
    pub hawl_satisfied: bool,
    /// Name of the wealth type for categorization.
    pub wealth_type_name: String,
}

impl CustomAsset {
    pub fn new(
        label: impl Into<String>,
        value: impl crate::inputs::IntoZakatDecimal,
        rate: impl crate::inputs::IntoZakatDecimal,
        nisab_threshold: impl crate::inputs::IntoZakatDecimal
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            value: value.into_zakat_decimal().unwrap_or(Decimal::ZERO),
            rate: rate.into_zakat_decimal().unwrap_or(Decimal::ZERO),
            nisab_threshold: nisab_threshold.into_zakat_decimal().unwrap_or(Decimal::ZERO),
            hawl_satisfied: true,
            wealth_type_name: "Custom".to_string()
        }
    }

    pub fn with_hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }
}

impl CalculateZakat for CustomAsset {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        let config_cow = config.resolve_config();
        let _config_ref = config_cow.as_ref();

        let wealth_type = crate::types::WealthType::Other(self.wealth_type_name.clone());
        
        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(
                self.nisab_threshold, 
                wealth_type, 
                "Hawl (1 lunar year) not met"
            ).with_label(self.label.clone()));
        }

        // Custom logic: simple Value * Rate check against Threshold
        // Assuming value is net assets for simplicity, or we can assume 0 liabilities.
        // For 'Generic Asset', let's stick to simplest: Value is Net.
        
        let liabilities = Decimal::ZERO;
        
        // Use ZakatDetails builder logic
        Ok(ZakatDetails::new(
            self.value,
            liabilities,
            self.nisab_threshold,
            self.rate,
            wealth_type
        ).with_label(self.label.clone()))
    }

    fn get_label(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn get_id(&self) -> Uuid {
        self.id
    }
}

/// A wrapper enum for all zakatable asset types.
/// This enables serialization and uniform handling in a portfolio.
/// 
/// Each variant represents a different category of zakatable wealth:
/// - `Business`: Trade goods and business assets (Urud al-Tijarah).
/// - `Income`: Professional income (Zakat al-Mustafad).
/// - `Livestock`: Animals (camels, cattle, sheep/goats).
/// - `Agriculture`: Crops and harvests.
/// - `Investment`: Stocks, crypto, and mutual funds.
/// - `Mining`: Mineral extraction (Ma'dan).
/// - `PreciousMetals`: Gold and silver.
/// - `Fitrah`: Zakat al-Fitr (obligatory charity at end of Ramadan).
/// - `Custom`: User-defined assets with custom rules.
/// 
/// Note: The `Ledger` variant is provided by the `zakat-ledger` crate.
/// When using the full zakat crate (facade), you can use `ExtendedPortfolioItem`
/// which includes ledger assets.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[typeshare::typeshare]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum PortfolioItem {
    /// Business assets (cash, inventory, receivables).
    Business(BusinessZakat),
    /// Professional income (salary, freelance earnings).
    Income(IncomeZakatCalculator),
    /// Livestock assets (camels, cattle, sheep, goats).
    Livestock(LivestockAssets),
    /// Agricultural assets (crops, harvests).
    Agriculture(AgricultureAssets),
    /// Investment assets (stocks, crypto, mutual funds).
    Investment(InvestmentAssets),
    /// Mining assets (minerals, extracted resources).
    Mining(MiningAssets),
    /// Precious metals (gold, silver).
    PreciousMetals(PreciousMetals),
    /// Zakat al-Fitr calculator.
    Fitrah(FitrahCalculator),
    /// User-defined custom assets.
    Custom(CustomAsset),
}

impl CalculateZakat for PortfolioItem {
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();

        match self {
            PortfolioItem::Business(asset) => asset.calculate_zakat(config),
            PortfolioItem::Income(asset) => asset.calculate_zakat(config),
            PortfolioItem::Livestock(asset) => asset.calculate_zakat(config),
            PortfolioItem::Agriculture(asset) => asset.calculate_zakat(config),
            PortfolioItem::Investment(asset) => asset.calculate_zakat(config),
            PortfolioItem::Mining(asset) => asset.calculate_zakat(config),
            PortfolioItem::PreciousMetals(asset) => asset.calculate_zakat(config),
            PortfolioItem::Fitrah(asset) => asset.calculate_zakat(config),
            PortfolioItem::Custom(asset) => asset.calculate_zakat(config),
        }
    }

    fn get_label(&self) -> Option<String> {
        match self {
            PortfolioItem::Business(asset) => asset.get_label(),
            PortfolioItem::Income(asset) => asset.get_label(),
            PortfolioItem::Livestock(asset) => asset.get_label(),
            PortfolioItem::Agriculture(asset) => asset.get_label(),
            PortfolioItem::Investment(asset) => asset.get_label(),
            PortfolioItem::Mining(asset) => asset.get_label(),
            PortfolioItem::PreciousMetals(asset) => asset.get_label(),
            PortfolioItem::Fitrah(asset) => asset.get_label(),
            PortfolioItem::Custom(asset) => asset.get_label(),
        }
    }

    fn get_id(&self) -> uuid::Uuid {
        match self {
            PortfolioItem::Business(asset) => asset.get_id(),
            PortfolioItem::Income(asset) => asset.get_id(),
            PortfolioItem::Livestock(asset) => asset.get_id(),
            PortfolioItem::Agriculture(asset) => asset.get_id(),
            PortfolioItem::Investment(asset) => asset.get_id(),
            PortfolioItem::Mining(asset) => asset.get_id(),
            PortfolioItem::PreciousMetals(asset) => asset.get_id(),
            PortfolioItem::Fitrah(asset) => asset.get_id(),
            PortfolioItem::Custom(asset) => asset.get_id(),
        }
    }
}

// Implement From<T> for each variant to simplify API usage

impl From<BusinessZakat> for PortfolioItem {
    fn from(asset: BusinessZakat) -> Self {
        PortfolioItem::Business(asset)
    }
}

impl From<IncomeZakatCalculator> for PortfolioItem {
    fn from(asset: IncomeZakatCalculator) -> Self {
        PortfolioItem::Income(asset)
    }
}

impl From<LivestockAssets> for PortfolioItem {
    fn from(asset: LivestockAssets) -> Self {
        PortfolioItem::Livestock(asset)
    }
}

impl From<AgricultureAssets> for PortfolioItem {
    fn from(asset: AgricultureAssets) -> Self {
        PortfolioItem::Agriculture(asset)
    }
}

impl From<InvestmentAssets> for PortfolioItem {
    fn from(asset: InvestmentAssets) -> Self {
        PortfolioItem::Investment(asset)
    }
}

impl From<MiningAssets> for PortfolioItem {
    fn from(asset: MiningAssets) -> Self {
        PortfolioItem::Mining(asset)
    }
}

impl From<PreciousMetals> for PortfolioItem {
    fn from(asset: PreciousMetals) -> Self {
        PortfolioItem::PreciousMetals(asset)
    }
}

impl From<FitrahCalculator> for PortfolioItem {
    fn from(asset: FitrahCalculator) -> Self {
        PortfolioItem::Fitrah(asset)
    }
}

impl From<CustomAsset> for PortfolioItem {
    fn from(asset: CustomAsset) -> Self {
        PortfolioItem::Custom(asset)
    }
}
