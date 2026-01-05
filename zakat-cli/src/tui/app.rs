//! Application state and screen management.

use rust_decimal::Decimal;

use tui_input::Input;
use zakat_core::assets::PortfolioItem;
use zakat_core::prelude::*;
use zakat_core::traits::CalculateZakat;
use zakat_providers::Prices;

/// Current screen/view in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Main menu screen
    Main,
    /// Adding a new asset
    AddAsset(AssetTypeSelection),
    /// Editing an existing asset
    EditAsset(usize),
    /// Viewing calculation results
    Results,
    /// Price loading screen
    Loading,
    /// Help screen
    Help,
}

/// Asset type selection for add asset flow
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AssetTypeSelection {
    #[default]
    Menu,
    Business,
    Gold,
    Silver,
    Cash,
    Investment,
    Agriculture,
}

/// Input field currently being edited
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InputField {
    #[default]
    None,
    Label,
    Amount,
    Weight,
    Inventory,
    Receivables,
    Liabilities,
    Filename,
}

/// Main application state
pub struct App {
    /// Whether the app should keep running
    pub running: bool,
    /// Current screen being displayed
    pub screen: Screen,
    /// User's portfolio of assets
    pub portfolio: ZakatPortfolio,
    /// Current metal prices
    pub prices: Option<Prices>,
    /// Zakat configuration
    pub config: ZakatConfig,
    /// Calculation results
    pub results: Option<PortfolioResult>,
    /// Selected menu item index
    pub menu_index: usize,
    /// Selected asset index in portfolio
    pub asset_index: usize,
    /// Text input widget state
    pub input: Input,
    /// Current input field being edited
    pub input_field: InputField,
    /// Status message to display
    pub message: Option<(String, MessageType)>,
    /// Form data being edited
    pub form_data: FormData,
    /// Index of asset being edited (if any)
    pub editing_asset_index: Option<usize>,
}

/// Type of status message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Form data for asset editing
#[derive(Debug, Clone, Default)]
pub struct FormData {
    pub label: String,
    pub amount: Decimal,
    pub weight: Decimal,
    pub inventory: Decimal,
    pub receivables: Decimal,
    pub liabilities: Decimal,
    pub is_gold: bool,
    pub is_irrigated: bool,
}

impl App {
    /// Create a new App instance with default settings
    pub fn new(_offline: bool) -> Self {
        Self {
            running: true,
            screen: Screen::Loading,
            portfolio: ZakatPortfolio::new(),
            prices: None,
            config: ZakatConfig::new()
                .with_madhab(Madhab::Hanafi)
                .with_nisab_standard(NisabStandard::Gold)
                .with_currency_code("USD"),
            results: None,
            menu_index: 0,
            asset_index: 0,
            input: Input::default(),
            input_field: InputField::None,
            message: None,
            form_data: FormData::default(),
            editing_asset_index: None,
        }
    }

    /// Set prices and update config
    pub fn set_prices(&mut self, prices: Prices) {
        self.config = self.config.clone()
            .with_gold_price(prices.gold_per_gram)
            .with_silver_price(prices.silver_per_gram);
        self.prices = Some(prices);
        self.screen = Screen::Main;
    }



    /// Get menu items for main screen
    pub fn main_menu_items(&self) -> Vec<&'static str> {
        vec![
            "[+] Add Asset",
            "[*] Edit Asset",
            "[S] Save Portfolio",
            "[L] Load Portfolio",
            "[=] Calculate Zakat",
            "[?] Help",
            "[Q] Quit",
        ]
    }

    /// Get asset type items for add asset flow
    pub fn asset_type_items(&self) -> Vec<&'static str> {
        vec![
            "[B] Business Assets",
            "[G] Gold",
            "[S] Silver",
            "[C] Cash/Savings",
            "[I] Investments",
            "[A] Agriculture",
            "[<] Back",
        ]
    }

    /// Move menu selection up
    pub fn menu_up(&mut self) {
        let max = match &self.screen {
            Screen::Main => self.main_menu_items().len(),
            Screen::AddAsset(AssetTypeSelection::Menu) => self.asset_type_items().len(),
            Screen::Results => self.portfolio.get_items().len().max(1),
            _ => 1,
        };
        if self.menu_index > 0 {
            self.menu_index -= 1;
        } else {
            self.menu_index = max.saturating_sub(1);
        }
    }

    /// Move menu selection down
    pub fn menu_down(&mut self) {
        let max = match &self.screen {
            Screen::Main => self.main_menu_items().len(),
            Screen::AddAsset(AssetTypeSelection::Menu) => self.asset_type_items().len(),
            Screen::Results => self.portfolio.get_items().len().max(1),
            _ => 1,
        };
        if self.menu_index < max.saturating_sub(1) {
            self.menu_index += 1;
        } else {
            self.menu_index = 0;
        }
    }

    /// Handle menu selection on main screen
    pub fn select_main_menu(&mut self) {
        match self.menu_index {
            0 => {
                // Add Asset
                self.screen = Screen::AddAsset(AssetTypeSelection::Menu);
                self.menu_index = 0;
            }
            1 => {
                // Edit Asset
                if !self.portfolio.get_items().is_empty() {
                    self.asset_index = 0;
                    self.screen = Screen::EditAsset(0);
                } else {
                    self.message = Some((
                        "No assets to edit. Add an asset first.".to_string(),
                        MessageType::Warning,
                    ));
                }
            }
            2 => {
                // Save Portfolio - start filename input
                self.input = Input::default().with_value("portfolio.json".to_string());
                self.input_field = InputField::Filename;
                self.message = Some((
                    "Enter filename and press Enter to save".to_string(),
                    MessageType::Info,
                ));
            }
            3 => {
                // Load Portfolio - start filename input
                self.input = Input::default().with_value("portfolio.json".to_string());
                self.input_field = InputField::Filename;
                self.message = Some((
                    "Enter filename and press Enter to load".to_string(),
                    MessageType::Info,
                ));
            }
            4 => {
                // Calculate
                if !self.portfolio.get_items().is_empty() {
                    let result = self.portfolio.calculate_total(&self.config);
                    self.results = Some(result);
                    self.screen = Screen::Results;
                    self.menu_index = 0;
                } else {
                    self.message = Some((
                        "No assets to calculate. Add assets first.".to_string(),
                        MessageType::Warning,
                    ));
                }
            }
            5 => {
                // Help
                self.screen = Screen::Help;
            }
            6 => {
                // Quit
                self.running = false;
            }
            _ => {}
        }
    }

    /// Start editing an asset
    pub fn start_editing(&mut self) {
        let items = self.portfolio.get_items();
        if self.asset_index >= items.len() {
            return;
        }
        
        // Populate form data from asset
        let asset = &items[self.asset_index];
        self.form_data = FormData::default();
        self.editing_asset_index = Some(self.asset_index);
        
        match asset {
            PortfolioItem::Business(b) => {
                self.form_data.label = b.label.clone().unwrap_or_default();
                self.form_data.amount = b.cash_on_hand;
                self.form_data.inventory = b.inventory_value;
                self.form_data.receivables = b.receivables;
                self.form_data.liabilities = b.total_liabilities();
                
                self.screen = Screen::AddAsset(AssetTypeSelection::Business);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value(self.form_data.label.clone());
            }
            PortfolioItem::PreciousMetals(pm) => {
                self.form_data.label = pm.label.clone().unwrap_or_default();
                self.form_data.weight = pm.weight_grams;
                self.form_data.is_gold = pm.metal_type == Some(WealthType::Gold);
                
                if self.form_data.is_gold {
                    self.screen = Screen::AddAsset(AssetTypeSelection::Gold);
                    self.input = Input::default().with_value(self.form_data.label.clone());
                } else {
                    self.screen = Screen::AddAsset(AssetTypeSelection::Silver);
                    self.input = Input::default().with_value(self.form_data.label.clone());
                }
                self.input_field = InputField::Label;
            }
            PortfolioItem::Investment(inv) => {
                self.form_data.label = inv.label.clone().unwrap_or_default();
                self.form_data.amount = inv.value;
                
                self.screen = Screen::AddAsset(AssetTypeSelection::Investment);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value(self.form_data.label.clone());
            }
            PortfolioItem::Agriculture(agr) => {
                self.form_data.label = agr.label.clone().unwrap_or_default();
                self.form_data.weight = agr.harvest_weight_kg;
                self.form_data.is_irrigated = agr.irrigation == IrrigationMethod::Irrigated;
                
                self.screen = Screen::AddAsset(AssetTypeSelection::Agriculture);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value(self.form_data.label.clone());
            }
            // For other types, we just support simplified Cash editing or treat as custom
            _ => {
                 // Try to map to cash/savings if possible, otherwise generic
                 self.form_data.label = CalculateZakat::get_label(asset).unwrap_or_default();
                 if let PortfolioItem::Income(inc) = asset {
                     self.form_data.amount = inc.income;
                     self.screen = Screen::AddAsset(AssetTypeSelection::Cash);
                 } else {
                     // Default to Cash form for generic/custom assets for now
                     self.screen = Screen::AddAsset(AssetTypeSelection::Cash);
                 }
                 self.input_field = InputField::Label;
                 self.input = Input::default().with_value(self.form_data.label.clone());
            }
        }
        
        self.message = Some(("Editing asset...".to_string(), MessageType::Info));
    }

    /// Handle asset type selection
    pub fn select_asset_type(&mut self) {
        self.editing_asset_index = None; // Reset editing state when selecting new asset type manually
        self.form_data = FormData::default();
        match self.menu_index {
            0 => {
                // Business
                self.screen = Screen::AddAsset(AssetTypeSelection::Business);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Business".to_string());
            }
            1 => {
                // Gold
                self.form_data.is_gold = true;
                self.screen = Screen::AddAsset(AssetTypeSelection::Gold);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Gold".to_string());
            }
            2 => {
                // Silver
                self.form_data.is_gold = false;
                self.screen = Screen::AddAsset(AssetTypeSelection::Silver);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Silver".to_string());
            }
            3 => {
                // Cash
                self.screen = Screen::AddAsset(AssetTypeSelection::Cash);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Savings".to_string());
            }
            4 => {
                // Investment
                self.screen = Screen::AddAsset(AssetTypeSelection::Investment);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Investments".to_string());
            }
            5 => {
                // Agriculture
                self.screen = Screen::AddAsset(AssetTypeSelection::Agriculture);
                self.input_field = InputField::Label;
                self.input = Input::default().with_value("Harvest".to_string());
            }
            6 => {
                // Back
                self.go_back();
            }
            _ => {}
        }
        self.menu_index = 0;
    }

    /// Go back to previous screen
    pub fn go_back(&mut self) {
        self.input_field = InputField::None;
        self.message = None;
        match &self.screen {
            Screen::AddAsset(AssetTypeSelection::Menu) => {
                self.screen = Screen::Main;
            }
            Screen::AddAsset(_) => {
                self.screen = Screen::AddAsset(AssetTypeSelection::Menu);
            }
            Screen::EditAsset(_) | Screen::Results | Screen::Help => {
                self.screen = Screen::Main;
            }
            _ => {}
        }
        self.menu_index = 0;
    }

    /// Add asset from current form data
    pub fn add_current_asset(&mut self) {
        let new_asset = match &self.screen {
            Screen::AddAsset(AssetTypeSelection::Business) => {
                let asset = BusinessZakat::new()
                    .label(&self.form_data.label)
                    .cash(self.form_data.amount)
                    .inventory(self.form_data.inventory)
                    .receivables(self.form_data.receivables)
                    .add_liability("Liabilities", self.form_data.liabilities);
                Some(PortfolioItem::Business(asset))
            }
            Screen::AddAsset(AssetTypeSelection::Gold) => {
                let asset = PreciousMetals::gold(self.form_data.weight)
                    .label(&self.form_data.label);
                Some(PortfolioItem::PreciousMetals(asset))
            }
            Screen::AddAsset(AssetTypeSelection::Silver) => {
                let asset = PreciousMetals::silver(self.form_data.weight)
                    .label(&self.form_data.label);
                Some(PortfolioItem::PreciousMetals(asset))
            }
            Screen::AddAsset(AssetTypeSelection::Cash) => {
                let asset = BusinessZakat::new()
                    .label(&self.form_data.label)
                    .cash(self.form_data.amount);
                Some(PortfolioItem::Business(asset))
            }
            Screen::AddAsset(AssetTypeSelection::Investment) => {
                let asset = InvestmentAssets::new()
                    .label(&self.form_data.label)
                    .value(self.form_data.amount)
                    .kind(InvestmentType::Stock);
                Some(PortfolioItem::Investment(asset))
            }
            Screen::AddAsset(AssetTypeSelection::Agriculture) => {
                let method = if self.form_data.is_irrigated {
                    IrrigationMethod::Irrigated
                } else {
                    IrrigationMethod::Rain
                };
                let asset = AgricultureAssets::new()
                    .label(&self.form_data.label)
                    .harvest_weight(self.form_data.weight)
                    .irrigation(method);
                Some(PortfolioItem::Agriculture(asset))
            }
            _ => None,
        };
        
        if let Some(asset) = new_asset {
            if let Some(index) = self.editing_asset_index {
                // Replace existing asset at index
                // Since ZakatPortfolio is immutable-ish (returns new instance), we need to handle this carefully
                // We'll rebuild the portfolio
                let old_items = self.portfolio.get_items();
                let mut new_portfolio: ZakatPortfolio = ZakatPortfolio::new();
                for (i, item) in old_items.iter().enumerate() {
                    if i == index {
                        new_portfolio = new_portfolio.add(asset.clone());
                    } else {
                        new_portfolio = new_portfolio.add(item.clone());
                    }
                }
                self.portfolio = new_portfolio;
                self.message = Some(("✓ Asset updated!".to_string(), MessageType::Success));
            } else {
                // Add new asset
                self.portfolio = self.portfolio.clone().add(asset);
                self.message = Some(("✓ New asset added!".to_string(), MessageType::Success));
            }
        }
        self.screen = Screen::Main;
        self.form_data = FormData::default();
        self.input_field = InputField::None;
    }

    /// Save portfolio to file
    pub fn save_portfolio(&mut self, filename: &str) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(&self.portfolio)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(filename, json)?;
        self.message = Some((
            format!("✓ Saved to {}", filename),
            MessageType::Success,
        ));
        self.input_field = InputField::None;
        Ok(())
    }

    /// Load portfolio from file
    pub fn load_portfolio(&mut self, filename: &str) -> std::io::Result<()> {
        let content = std::fs::read_to_string(filename)?;
        let portfolio: ZakatPortfolio = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.portfolio = portfolio;
        self.message = Some((
            format!("✓ Loaded from {}", filename),
            MessageType::Success,
        ));
        self.input_field = InputField::None;
        Ok(())
    }
}
