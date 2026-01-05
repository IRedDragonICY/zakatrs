//! Event handling for keyboard input using crossterm.

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use rust_decimal::Decimal;
use std::io;
use std::time::Duration;
use tui_input::backend::crossterm::EventHandler;
use zakat_core::prelude::ZakatPortfolio;

use crate::tui::app::{App, AssetTypeSelection, InputField, MessageType, Screen};

/// Poll for events and handle them.
/// Returns Ok(true) if the app should quit.
pub fn handle_events(app: &mut App) -> io::Result<bool> {
    // Poll for events with a small timeout
    if event::poll(Duration::from_millis(100))?
        && let Event::Key(key) = event::read()? {
            // Only handle key press events, not release
            if key.kind != KeyEventKind::Press {
                return Ok(false);
            }

            // Clear any existing message on key press
            if key.code != KeyCode::Enter {
                app.message = None;
            }

            // Handle Ctrl+C globally
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return Ok(true);
            }

            // If we're in input mode, handle text input
            if app.input_field != InputField::None {
                return handle_input_mode(app, key);
            }

            // Handle keys based on current screen
            match &app.screen {
                Screen::Loading => {
                    // No interaction during loading
                }
                Screen::Main => handle_main_screen(app, key.code),
                Screen::AddAsset(AssetTypeSelection::Menu) => handle_asset_menu(app, key.code),
                Screen::AddAsset(_) => handle_asset_form(app, key.code),
                Screen::EditAsset(_) => handle_edit_asset(app, key.code),
                Screen::Results => handle_results(app, key.code),
                Screen::Help => handle_help(app, key.code),
            }

            // Check if we should quit
            if !app.running {
                return Ok(true);
            }
        }

    Ok(false)
}

/// Handle keyboard input when in input mode (text fields)
fn handle_input_mode(app: &mut App, key: event::KeyEvent) -> io::Result<bool> {
    match key.code {
        KeyCode::Enter => {
            // Get the current input value
            let value = app.input.value().to_string();
            
            match app.input_field {
                InputField::Label => {
                    app.form_data.label = value;
                    // Move to next field based on asset type
                    match &app.screen {
                        Screen::AddAsset(AssetTypeSelection::Business) => {
                            app.input_field = InputField::Amount;
                            app.input = tui_input::Input::default();
                            app.message = Some(("Enter cash on hand amount".to_string(), MessageType::Info));
                        }
                        Screen::AddAsset(AssetTypeSelection::Gold | AssetTypeSelection::Silver) => {
                            app.input_field = InputField::Weight;
                            app.input = tui_input::Input::default();
                            app.message = Some(("Enter weight in grams".to_string(), MessageType::Info));
                        }
                        Screen::AddAsset(AssetTypeSelection::Cash | AssetTypeSelection::Investment) => {
                            app.input_field = InputField::Amount;
                            app.input = tui_input::Input::default();
                            app.message = Some(("Enter amount".to_string(), MessageType::Info));
                        }
                        Screen::AddAsset(AssetTypeSelection::Agriculture) => {
                            app.input_field = InputField::Weight;
                            app.input = tui_input::Input::default();
                            app.message = Some(("Enter harvest weight in kg".to_string(), MessageType::Info));
                        }
                        _ => {}
                    }
                }
                InputField::Amount => {
                    if let Ok(amount) = value.parse::<Decimal>() {
                        app.form_data.amount = amount;
                        match &app.screen {
                            Screen::AddAsset(AssetTypeSelection::Business) => {
                                app.input_field = InputField::Inventory;
                                app.input = tui_input::Input::default();
                                app.message = Some(("Enter inventory value".to_string(), MessageType::Info));
                            }
                            Screen::AddAsset(AssetTypeSelection::Cash | AssetTypeSelection::Investment) => {
                                // Done - add asset
                                app.add_current_asset();
                            }
                            _ => {}
                        }
                    } else {
                        app.message = Some(("Invalid number format".to_string(), MessageType::Error));
                    }
                }
                InputField::Weight => {
                    if let Ok(weight) = value.parse::<Decimal>() {
                        app.form_data.weight = weight;
                        match &app.screen {
                            Screen::AddAsset(AssetTypeSelection::Gold | AssetTypeSelection::Silver) => {
                                // Done - add asset
                                app.add_current_asset();
                            }
                            Screen::AddAsset(AssetTypeSelection::Agriculture) => {
                                // Toggle irrigation with message
                                app.input_field = InputField::None;
                                app.message = Some((
                                    "Press 'i' for irrigated, 'r' for rain-fed, Enter to save".to_string(),
                                    MessageType::Info,
                                ));
                            }
                            _ => {}
                        }
                    } else {
                        app.message = Some(("Invalid number format".to_string(), MessageType::Error));
                    }
                }
                InputField::Inventory => {
                    if let Ok(inv) = value.parse::<Decimal>() {
                        app.form_data.inventory = inv;
                        app.input_field = InputField::Receivables;
                        app.input = tui_input::Input::default();
                        app.message = Some(("Enter receivables amount".to_string(), MessageType::Info));
                    } else {
                        app.message = Some(("Invalid number format".to_string(), MessageType::Error));
                    }
                }
                InputField::Receivables => {
                    if let Ok(rec) = value.parse::<Decimal>() {
                        app.form_data.receivables = rec;
                        app.input_field = InputField::Liabilities;
                        app.input = tui_input::Input::default();
                        app.message = Some(("Enter liabilities amount".to_string(), MessageType::Info));
                    } else {
                        app.message = Some(("Invalid number format".to_string(), MessageType::Error));
                    }
                }
                InputField::Liabilities => {
                    if let Ok(liab) = value.parse::<Decimal>() {
                        app.form_data.liabilities = liab;
                        // Done - add business asset
                        app.add_current_asset();
                    } else {
                        app.message = Some(("Invalid number format".to_string(), MessageType::Error));
                    }
                }
                InputField::Filename => {
                    // Determine if save or load based on menu index
                    if app.menu_index == 2 {
                        // Save
                        if let Err(e) = app.save_portfolio(&value) {
                            app.message = Some((format!("Save error: {}", e), MessageType::Error));
                        }
                    } else {
                        // Load
                        if let Err(e) = app.load_portfolio(&value) {
                            app.message = Some((format!("Load error: {}", e), MessageType::Error));
                        }
                    }
                }
                InputField::None => {}
            }
        }
        KeyCode::Esc => {
            // Cancel current input and go back to field navigation
            app.input_field = InputField::None;
            app.input = tui_input::Input::default();
        }
        KeyCode::Tab | KeyCode::Down => {
            // Save current value and advance to next field
            save_current_field_value(app);
            advance_to_next_field(app);
        }
        KeyCode::BackTab | KeyCode::Up => {
            // Save current value and go to previous field
            save_current_field_value(app);
            go_to_previous_field(app);
        }
        _ => {
            // Pass the key event to tui-input
            app.input.handle_event(&Event::Key(key));
        }
    }

    Ok(false)
}

/// Save current input value to the appropriate form field
fn save_current_field_value(app: &mut App) {
    let value = app.input.value().to_string();
    
    match app.input_field {
        InputField::Label => {
            app.form_data.label = value;
        }
        InputField::Amount => {
            if let Ok(amount) = value.parse::<Decimal>() {
                app.form_data.amount = amount;
            }
        }
        InputField::Weight => {
            if let Ok(weight) = value.parse::<Decimal>() {
                app.form_data.weight = weight;
            }
        }
        InputField::Inventory => {
            if let Ok(inv) = value.parse::<Decimal>() {
                app.form_data.inventory = inv;
            }
        }
        InputField::Receivables => {
            if let Ok(rec) = value.parse::<Decimal>() {
                app.form_data.receivables = rec;
            }
        }
        InputField::Liabilities => {
            if let Ok(liab) = value.parse::<Decimal>() {
                app.form_data.liabilities = liab;
            }
        }
        _ => {}
    }
}

/// Handle main menu screen
fn handle_main_screen(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        KeyCode::Enter => app.select_main_menu(),
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('?') => app.screen = Screen::Help,
        _ => {}
    }
}

/// Handle asset type selection menu
fn handle_asset_menu(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        KeyCode::Enter => app.select_asset_type(),
        KeyCode::Esc => app.go_back(),
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

/// Handle asset form input
fn handle_asset_form(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.go_back(),
        KeyCode::Tab => {
            // Navigate to next field
            advance_to_next_field(app);
        }
        KeyCode::BackTab => {
            // Navigate to previous field
            go_to_previous_field(app);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            go_to_previous_field(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            advance_to_next_field(app);
        }
        KeyCode::Enter => {
            // If no field is active, save the asset
            if app.input_field == InputField::None {
                // Check if we have minimum data
                if !app.form_data.label.is_empty() {
                    app.add_current_asset();
                } else {
                    // Start editing label field
                    app.input_field = InputField::Label;
                    app.input = tui_input::Input::default();
                }
            } else {
                // A field is active - confirm and move to next (should be handled by input mode)
            }
        }
        KeyCode::Char('i') => {
            // Agriculture - set irrigated
            if matches!(app.screen, Screen::AddAsset(AssetTypeSelection::Agriculture)) {
                app.form_data.is_irrigated = true;
                app.message = Some(("Irrigation: Irrigated".to_string(), MessageType::Info));
            }
        }
        KeyCode::Char('r') => {
            // Agriculture - set rain-fed
            if matches!(app.screen, Screen::AddAsset(AssetTypeSelection::Agriculture)) {
                app.form_data.is_irrigated = false;
                app.message = Some(("Irrigation: Rain-fed".to_string(), MessageType::Info));
            }
        }
        _ => {}
    }
}

/// Get all fields for the current asset type
fn get_form_fields(screen: &Screen) -> Vec<InputField> {
    match screen {
        Screen::AddAsset(AssetTypeSelection::Business) => vec![
            InputField::Label,
            InputField::Amount,
            InputField::Inventory,
            InputField::Receivables,
            InputField::Liabilities,
        ],
        Screen::AddAsset(AssetTypeSelection::Gold) | Screen::AddAsset(AssetTypeSelection::Silver) => vec![
            InputField::Label,
            InputField::Weight,
        ],
        Screen::AddAsset(AssetTypeSelection::Cash) | Screen::AddAsset(AssetTypeSelection::Investment) => vec![
            InputField::Label,
            InputField::Amount,
        ],
        Screen::AddAsset(AssetTypeSelection::Agriculture) => vec![
            InputField::Label,
            InputField::Weight,
        ],
        _ => vec![InputField::Label],
    }
}

/// Advance to the next field in the form
fn advance_to_next_field(app: &mut App) {
    let fields = get_form_fields(&app.screen);
    
    if app.input_field == InputField::None {
        // Start with first field
        if let Some(first) = fields.first() {
            app.input_field = first.clone();
            prefill_input_with_current_value(app);
        }
    } else {
        // Find current field and move to next
        if let Some(pos) = fields.iter().position(|f| *f == app.input_field) {
            if pos + 1 < fields.len() {
                app.input_field = fields[pos + 1].clone();
                prefill_input_with_current_value(app);
            } else {
                // At end - deselect field (ready to save)
                app.input_field = InputField::None;
            }
        }
    }
}

/// Go to previous field in the form
fn go_to_previous_field(app: &mut App) {
    let fields = get_form_fields(&app.screen);
    
    if app.input_field == InputField::None {
        // Start with last field
        if let Some(last) = fields.last() {
            app.input_field = last.clone();
            prefill_input_with_current_value(app);
        }
    } else {
        // Find current field and move to previous
        if let Some(pos) = fields.iter().position(|f| *f == app.input_field) {
            if pos > 0 {
                app.input_field = fields[pos - 1].clone();
                prefill_input_with_current_value(app);
            }
        }
    }
}

/// Pre-fill the input with the current field's existing value
fn prefill_input_with_current_value(app: &mut App) {
    let value = match app.input_field {
        InputField::Label => app.form_data.label.clone(),
        InputField::Amount => {
            if app.form_data.amount > Decimal::ZERO {
                app.form_data.amount.to_string()
            } else {
                String::new()
            }
        }
        InputField::Weight => {
            if app.form_data.weight > Decimal::ZERO {
                app.form_data.weight.to_string()
            } else {
                String::new()
            }
        }
        InputField::Inventory => {
            if app.form_data.inventory > Decimal::ZERO {
                app.form_data.inventory.to_string()
            } else {
                String::new()
            }
        }
        InputField::Receivables => {
            if app.form_data.receivables > Decimal::ZERO {
                app.form_data.receivables.to_string()
            } else {
                String::new()
            }
        }
        InputField::Liabilities => {
            if app.form_data.liabilities > Decimal::ZERO {
                app.form_data.liabilities.to_string()
            } else {
                String::new()
            }
        }
        _ => String::new(),
    };
    
    app.input = tui_input::Input::default().with_value(value);
}

/// Handle edit asset screen
fn handle_edit_asset(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => {
            if app.asset_index > 0 {
                app.asset_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.portfolio.get_items().len();
            if app.asset_index < max.saturating_sub(1) {
                app.asset_index += 1;
            }
        }
        KeyCode::Enter => {
            // Edit selected asset
            app.start_editing();
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            // Delete selected asset
            let items = app.portfolio.get_items();
            if !items.is_empty() && app.asset_index < items.len() {
                // Note: ZakatPortfolio doesn't have a remove method, so we rebuild
                let mut new_portfolio = ZakatPortfolio::new();
                for (i, item) in items.iter().enumerate() {
                    if i != app.asset_index {
                        new_portfolio = new_portfolio.add(item.clone());
                    }
                }
                app.portfolio = new_portfolio;
                app.message = Some(("Asset deleted".to_string(), MessageType::Success));
                if app.asset_index > 0 {
                    app.asset_index -= 1;
                }
                if app.portfolio.get_items().is_empty() {
                    app.go_back();
                }
            }
        }
        _ => {}
    }
}

/// Handle results screen
fn handle_results(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => app.go_back(),
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        _ => {}
    }
}

/// Handle help screen
fn handle_help(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?') => {
            app.go_back();
        }
        _ => {}
    }
}
