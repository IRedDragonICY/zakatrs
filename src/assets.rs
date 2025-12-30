use serde::{Serialize, Deserialize};
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;

use crate::maal::business::BusinessZakat;
use crate::maal::income::IncomeZakatCalculator;
use crate::maal::livestock::LivestockAssets;
use crate::maal::agriculture::AgricultureAssets;
use crate::maal::investments::InvestmentAssets;
use crate::maal::mining::MiningAssets;
use crate::maal::precious_metals::PreciousMetals;
use crate::fitrah::FitrahCalculator;

/// A wrapper enum for all zakatable asset types.
/// This enables serialization and uniform handling in a portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PortfolioItem {
    Business(BusinessZakat),
    Income(IncomeZakatCalculator),
    Livestock(LivestockAssets),
    Agriculture(AgricultureAssets),
    Investment(InvestmentAssets),
    Mining(MiningAssets),
    PreciousMetals(PreciousMetals),
    Fitrah(FitrahCalculator), // Although usually separate, useful to track in user portfolio
}

impl CalculateZakat for PortfolioItem {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        match self {
            PortfolioItem::Business(asset) => asset.calculate_zakat(config),
            PortfolioItem::Income(asset) => asset.calculate_zakat(config),
            PortfolioItem::Livestock(asset) => asset.calculate_zakat(config),
            PortfolioItem::Agriculture(asset) => asset.calculate_zakat(config),
            PortfolioItem::Investment(asset) => asset.calculate_zakat(config),
            PortfolioItem::Mining(asset) => asset.calculate_zakat(config),
            PortfolioItem::PreciousMetals(asset) => asset.calculate_zakat(config),
            PortfolioItem::Fitrah(asset) => asset.calculate_zakat(config),
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
