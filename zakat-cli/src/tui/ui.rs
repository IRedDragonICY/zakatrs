//! UI rendering for the TUI (Modernized Premium Design).
//!
//! Features a premium Islamic Finance aesthetic with Gold/Emerald themes,
//! component-based architecture, and exceptional user experience.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, 
        Padding, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

use crate::tui::app::{App, AssetTypeSelection, InputField, MessageType, Screen};
use crate::tui::components::{LoadingSpinner, StatCard};
use crate::tui::theme::{icons, theme};

use zakat_core::assets::PortfolioItem;
use zakat_core::prelude::{PortfolioItemResult, WealthType};
use zakat_core::traits::CalculateZakat;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

// ═══════════════════════════════════════════════════════════════════════════
// MAIN UI ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════════

/// Main UI rendering function - entry point for all screen rendering.
pub fn ui(frame: &mut Frame, app: &App) {
    let t = theme();

    // Clear the entire frame first to prevent visual artifacts from popups
    frame.render_widget(Clear, frame.area());
    
    // Then set background color
    frame.render_widget(Block::default().style(t.bg()), frame.area());

    // Root Layout: Header | Main Content | Status Bar
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(frame.area());

    // 1. Render Header
    render_header(frame, root_layout[0], app);

    // 2. Render Content (with Sidebar)
    render_content(frame, root_layout[1], app);

    // 3. Render Status Bar
    render_status_bar(frame, root_layout[2], app);

    // 4. Overlays (Popups) - rendered last so they appear on top
    // Only show input popup for filename input (not for asset forms which have inline input)
    if app.input_field == InputField::Filename {
        render_input_popup(frame, app);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HEADER
// ═══════════════════════════════════════════════════════════════════════════

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let header_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(t.slate_light))
        .style(t.bg());

    let inner = header_block.inner(area);
    frame.render_widget(header_block, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(inner);

    // Left: Brand
    let brand = Line::from(vec![
        Span::raw(" "),
        Span::styled(icons::MOON, Style::default().fg(t.gold)),
        Span::raw(" "),
        Span::styled("ZAKAT", t.title()),
        Span::styled("CLI", Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD)),
    ]);
    frame.render_widget(
        Paragraph::new(brand).alignment(Alignment::Left),
        layout[0],
    );

    // Right: Price Ticker
    let prices_line = if let Some(prices) = &app.prices {
        Line::from(vec![
            Span::styled("Gold: ", Style::default().fg(t.gold)),
            Span::styled(format!("${:.2}/g", prices.gold_per_gram), Style::default().fg(t.text_primary)),
            Span::raw("  "),
            Span::styled(icons::SEPARATOR, Style::default().fg(t.slate_light)),
            Span::raw("  "),
            Span::styled("Silver: ", Style::default().fg(t.text_muted)),
            Span::styled(format!("${:.2}/g", prices.silver_per_gram), Style::default().fg(t.text_primary)),
            Span::raw("  "),
            Span::styled(icons::CHECK, Style::default().fg(t.success)),
            Span::styled(" Live", Style::default().fg(t.success)),
        ])
    } else {
        Line::from(vec![
            Span::styled("Fetching Market Data...", Style::default().fg(t.text_muted).add_modifier(Modifier::ITALIC)),
        ])
    };

    frame.render_widget(
        Paragraph::new(prices_line).alignment(Alignment::Right),
        layout[1],
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// MAIN CONTENT
// ═══════════════════════════════════════════════════════════════════════════

fn render_content(frame: &mut Frame, area: Rect, app: &App) {

    // Layout: Sidebar | Main Dashboard
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(26), // Sidebar width
            Constraint::Min(0),     // Main content
        ])
        .split(area);

    // Render Sidebar
    render_sidebar(frame, chunks[0], app);

    // Render Main Area based on Screen
    match &app.screen {
        Screen::Loading => render_loading(frame, chunks[1], app),
        Screen::Main => render_dashboard(frame, chunks[1], app),
        Screen::AddAsset(AssetTypeSelection::Menu) => {
            render_dashboard(frame, chunks[1], app);
            render_asset_picker(frame, frame.area(), app);
        }
        Screen::AddAsset(_asset_type) => {
            render_dashboard(frame, chunks[1], app);
            render_asset_form(frame, frame.area(), app);
        }
        Screen::EditAsset(_) => {
            render_dashboard(frame, chunks[1], app);
            render_edit_overlay(frame, frame.area(), app);
        }
        Screen::Results => render_results_report(frame, chunks[1], app),
        Screen::Help => {
            render_dashboard(frame, chunks[1], app);
            render_help(frame, frame.area());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIDEBAR
// ═══════════════════════════════════════════════════════════════════════════

fn render_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    // Menu items with icons
    let menu_data: [(& str, &str); 7] = [
        (icons::ADD, "Add Asset"),
        (icons::EDIT, "Edit Asset"),
        (icons::SAVE, "Save Portfolio"),
        (icons::FOLDER, "Load Portfolio"),
        (icons::CALCULATOR, "Calculate"),
        (icons::HELP, "Help"),
        (icons::CLOSE, "Quit"),
    ];

    let menu_items: Vec<ListItem> = menu_data
        .iter()
        .enumerate()
        .map(|(i, (icon, text))| {
            let is_active = match app.screen {
                Screen::Main => i == app.menu_index,
                Screen::AddAsset(_) => i == 0,
                Screen::EditAsset(_) => i == 1,
                Screen::Results => i == 4,
                Screen::Help => i == 5,
                _ => false,
            };

            let style = if is_active {
                t.highlight()
            } else {
                Style::default().fg(t.text_muted)
            };

            let indicator = if is_active { icons::ARROW_RIGHT } else { " " };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", indicator), style),
                Span::styled(format!("{} ", icon), style),
                Span::styled(*text, style),
            ]))
            .style(style)
        })
        .collect();

    let list = List::new(menu_items).block(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(t.slate_light))
            .style(t.bg())
            .padding(Padding::new(0, 1, 1, 1)),
    );

    frame.render_widget(list, area);
}

// ═══════════════════════════════════════════════════════════════════════════
// DASHBOARD
// ═══════════════════════════════════════════════════════════════════════════

fn render_dashboard(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    // Clear and fill the entire dashboard area to prevent artifacts from popups
    frame.render_widget(Clear, area);
    frame.render_widget(Block::default().style(t.bg()), area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Stats row
            Constraint::Min(0),    // Portfolio table
        ])
        .split(area);


    // Stats Cards Layout
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .horizontal_margin(1)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(layout[0]);

    // Calculate totals for display
    let total_value: Decimal = app
        .portfolio
        .get_items()
        .iter()
        .map(|item| calculate_item_value(item, app))
        .sum();

    let nisab_threshold = if let Some(prices) = &app.prices {
        prices.gold_per_gram * dec!(85.0)
    } else {
        Decimal::ZERO
    };

    // Stat Card 1: Total Assets
    StatCard::new("Est. Total Assets", &format!("${:.2}", total_value))
        .value_color(t.text_primary)
        .render(frame, stats_layout[0]);

    // Stat Card 2: Nisab Threshold
    if app.prices.is_some() {
        StatCard::new("Nisab Threshold", &format!("${:.2}", nisab_threshold))
            .value_color(t.gold)
            .subtitle("Based on gold")
            .render(frame, stats_layout[1]);
    } else {
        StatCard::new("Nisab Threshold", "Loading...")
            .value_color(t.text_muted)
            .render(frame, stats_layout[1]);
    }

    // Stat Card 3: Status
    if app.prices.is_some() {
        let (status, color) = if total_value >= nisab_threshold && total_value > Decimal::ZERO {
            ("Likely Payable", t.success)
        } else if total_value > Decimal::ZERO {
            ("Below Nisab", t.text_muted)
        } else {
            ("No Assets", t.text_muted)
        };
        StatCard::new("Nisab Status", status)
            .value_color(color)
            .render(frame, stats_layout[2]);
    } else {
        StatCard::new("Status", "Waiting...")
            .value_color(t.text_muted)
            .render(frame, stats_layout[2]);
    }

    // Portfolio Table
    render_portfolio_table(frame, layout[1], app);
}

fn calculate_item_value(item: &PortfolioItem, app: &App) -> Decimal {
    match item {
        PortfolioItem::Business(b) => {
            b.cash_on_hand + b.inventory_value + b.receivables - b.total_liabilities()
        }
        PortfolioItem::PreciousMetals(pm) => {
            let price = if pm.metal_type == Some(WealthType::Gold) {
                app.prices.as_ref().map(|p| p.gold_per_gram).unwrap_or_default()
            } else {
                app.prices.as_ref().map(|p| p.silver_per_gram).unwrap_or_default()
            };
            pm.weight_grams * price
        }
        PortfolioItem::Investment(inv) => inv.value,
        PortfolioItem::Income(inc) => inc.income,
        _ => Decimal::ZERO,
    }
}

fn render_portfolio_table(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();
    let items = app.portfolio.get_items();

    // Empty state
    if items.is_empty() {
        let center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Length(4),
                Constraint::Percentage(30),
            ])
            .split(area);

        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Your portfolio is empty",
                Style::default().fg(t.text_muted),
            )),
            Line::from(Span::styled(
                "Select 'Add Asset' to begin tracking your wealth",
                Style::default().fg(t.gold),
            )),
        ])
        .alignment(Alignment::Center);

        frame.render_widget(msg, center[1]);
        return;
    }

    // Build table rows
    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let label = CalculateZakat::get_label(item).unwrap_or_else(|| format!("Item #{}", i + 1));
            let (icon, type_color) = get_asset_icon_and_color(item);
            let value = calculate_item_value(item, app);

            let is_selected = matches!(app.screen, Screen::EditAsset(_)) && app.asset_index == i;
            let row_style = if is_selected {
                Style::default().bg(t.slate_light).fg(t.text_primary)
            } else {
                Style::default().fg(t.text_primary)
            };

            Row::new(vec![
                Cell::from(format!(" {} ", icon)),
                Cell::from(label).style(Style::default().fg(type_color).add_modifier(Modifier::BOLD)),
                Cell::from(format!("${:.2}", value)).style(Style::default().fg(t.text_primary)),
            ])
            .style(row_style)
            .height(2)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(55),
            Constraint::Percentage(35),
        ],
    )
    .header(
        Row::new(vec!["", "ASSET", "VALUE"])
            .style(Style::default().fg(t.text_muted).add_modifier(Modifier::UNDERLINED)),
    )
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(t.slate_light))
            .title(" Portfolio Assets ")
            .title_style(Style::default().fg(t.text_muted))
            .padding(Padding::new(1, 1, 0, 0)),
    );

    frame.render_widget(table, area);
}

fn get_asset_icon_and_color(item: &PortfolioItem) -> (&'static str, ratatui::style::Color) {
    let t = theme();
    match item {
        PortfolioItem::Business(_) => (icons::BUILDING, t.accent),
        PortfolioItem::PreciousMetals(pm) => {
            if pm.metal_type == Some(WealthType::Gold) {
                (icons::GOLD, t.gold)
            } else {
                (icons::SILVER, t.asset_silver())
            }
        }
        PortfolioItem::Investment(_) => (icons::CHART, t.success),
        PortfolioItem::Agriculture(_) => (icons::GRAIN, t.asset_agriculture()),
        PortfolioItem::Livestock(_) => (icons::LIVESTOCK, t.warning),
        PortfolioItem::Income(_) => (icons::CASH, t.accent),
        _ => (icons::PACKAGE, t.text_muted),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LOADING SCREEN
// ═══════════════════════════════════════════════════════════════════════════

fn render_loading(frame: &mut Frame, area: Rect, _app: &App) {
    let t = theme();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.slate_light))
        .style(t.bg());

    frame.render_widget(block.clone(), area);

    // Use spinner animation - use a simple frame counter based on time
    let frame_idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 200) as usize;

    LoadingSpinner::new("Fetching live market prices...")
        .frame(frame_idx)
        .render(frame, block.inner(area));
}

// ═══════════════════════════════════════════════════════════════════════════
// ASSET PICKER (POPUP)
// ═══════════════════════════════════════════════════════════════════════════

fn render_asset_picker(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    // Compact centered popup
    let popup_area = centered_rect(50, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Select Asset Type ")
        .title_alignment(Alignment::Center)
        .title_style(t.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(t.border_active())
        .style(t.bg());

    frame.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);
    
    // Single column list layout - using consistent-width emojis
    // Note: Some emojis have varying widths in terminals, so we pad all icons to 3 chars
    let options: [(&str, &str, &str); 7] = [
        ("🏢 ", "Business Assets", "Trade goods, cash, receivables"),
        ("🪙 ", "Gold", "Jewelry, bars, stored wealth"),
        ("🥈 ", "Silver", "Utensils, coins, savings"),
        ("💵 ", "Cash / Savings", "Bank accounts, cash on hand"),
        ("📈 ", "Investments", "Stocks, Crypto, Mutual Funds"),
        ("🌾 ", "Agriculture", "Crops, harvest, produce"),
        ("←  ", "Back", "Return to main menu"),
    ];

    // Create list items
    let list_items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, (icon, title, desc))| {
            let is_selected = i == app.menu_index;

            let (arrow, style) = if is_selected {
                (icons::ARROW_RIGHT, t.highlight())
            } else {
                (" ", Style::default().fg(t.text_primary))
            };

            let desc_style = if is_selected {
                Style::default().fg(t.slate)
            } else {
                Style::default().fg(t.text_muted)
            };

            // Use format! to create owned strings that avoid lifetime issues
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(format!(" {} ", arrow), style),
                    Span::styled(icon.to_string(), style),
                    Span::styled(*title, style.add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw("       "),
                    Span::styled(*desc, desc_style),
                ]),
            ])
            .style(if is_selected { t.highlight() } else { Style::default() })
        })
        .collect();

    let content_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let list = List::new(list_items)
        .block(Block::default().padding(Padding::new(1, 1, 0, 0)));

    frame.render_widget(list, content_area[0]);

    // Footer hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("[↑↓] ", t.accent_style()),
        Span::raw("Navigate  "),
        Span::styled("[Enter] ", t.accent_style()),
        Span::raw("Select  "),
        Span::styled("[Esc] ", t.accent_style()),
        Span::raw("Cancel"),
    ]))
    .alignment(Alignment::Center);

    frame.render_widget(hint, content_area[1]);
}

// ═══════════════════════════════════════════════════════════════════════════
// ASSET FORM (POPUP)
// ═══════════════════════════════════════════════════════════════════════════

fn render_asset_form(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let asset_type_name = match &app.screen {
        Screen::AddAsset(AssetTypeSelection::Business) => "Business Asset",
        Screen::AddAsset(AssetTypeSelection::Gold) => "Gold",
        Screen::AddAsset(AssetTypeSelection::Silver) => "Silver",
        Screen::AddAsset(AssetTypeSelection::Cash) => "Cash / Savings",
        Screen::AddAsset(AssetTypeSelection::Investment) => "Investment",
        Screen::AddAsset(AssetTypeSelection::Agriculture) => "Agriculture",
        _ => "New Asset",
    };

    let title = if app.editing_asset_index.is_some() {
        format!(" Edit {} ", asset_type_name)
    } else {
        format!(" Add {} ", asset_type_name)
    };

    let popup_area = centered_rect(65, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .title_style(t.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(t.border_active())
        .style(t.bg());

    frame.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);
    
    // Get fields for this asset type
    let fields = get_asset_fields(&app.screen);
    
    // Calculate layout - each field gets 2 lines + spacing
    let field_count = fields.len();
    let mut constraints: Vec<Constraint> = Vec::new();
    for _ in 0..field_count {
        constraints.push(Constraint::Length(2)); // Field row
    }
    constraints.push(Constraint::Length(1)); // Spacer
    constraints.push(Constraint::Length(2)); // Buttons
    constraints.push(Constraint::Min(0));    // Remaining space
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(constraints)
        .split(inner);

    // Render each field
    for (i, (field_type, label, unit)) in fields.iter().enumerate() {
        let is_active = app.input_field == *field_type;
        let value = get_field_value(app, field_type);
        
        render_form_field(
            frame,
            chunks[i],
            label,
            &value,
            unit,
            is_active,
            if is_active { Some(app.input.value()) } else { None },
            t,
        );
    }

    // Action buttons / Instructions
    let button_area = chunks[field_count + 1];
    
    // Create button row with OK and Cancel
    let button_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ])
        .split(button_area);

    // OK Button
    let ok_style = Style::default()
        .fg(t.slate)
        .bg(t.emerald)
        .add_modifier(Modifier::BOLD);
    frame.render_widget(
        Paragraph::new(" ✓ OK [Enter] ")
            .style(ok_style)
            .alignment(Alignment::Center),
        button_layout[1],
    );

    // Cancel Button  
    let cancel_style = Style::default()
        .fg(t.text_primary)
        .bg(t.slate_light);
    frame.render_widget(
        Paragraph::new(" ✕ Cancel [Esc] ")
            .style(cancel_style)
            .alignment(Alignment::Center),
        button_layout[2],
    );
}

/// Get the fields needed for each asset type
fn get_asset_fields(screen: &Screen) -> Vec<(InputField, &'static str, &'static str)> {
    match screen {
        Screen::AddAsset(AssetTypeSelection::Business) => vec![
            (InputField::Label, "Label", ""),
            (InputField::Amount, "Cash on Hand", "$"),
            (InputField::Inventory, "Inventory Value", "$"),
            (InputField::Receivables, "Receivables", "$"),
            (InputField::Liabilities, "Liabilities", "$"),
        ],
        Screen::AddAsset(AssetTypeSelection::Gold) | Screen::AddAsset(AssetTypeSelection::Silver) => vec![
            (InputField::Label, "Label", ""),
            (InputField::Weight, "Weight", "grams"),
        ],
        Screen::AddAsset(AssetTypeSelection::Cash) | Screen::AddAsset(AssetTypeSelection::Investment) => vec![
            (InputField::Label, "Label", ""),
            (InputField::Amount, "Amount", "$"),
        ],
        Screen::AddAsset(AssetTypeSelection::Agriculture) => vec![
            (InputField::Label, "Label", ""),
            (InputField::Weight, "Harvest Weight", "kg"),
        ],
        _ => vec![
            (InputField::Label, "Label", ""),
            (InputField::Amount, "Amount", "$"),
        ],
    }
}

/// Get the current value for a field
fn get_field_value(app: &App, field: &InputField) -> String {
    match field {
        InputField::Label => app.form_data.label.clone(),
        InputField::Amount => {
            if app.form_data.amount > Decimal::ZERO {
                format!("{}", app.form_data.amount)
            } else {
                String::new()
            }
        }
        InputField::Weight => {
            if app.form_data.weight > Decimal::ZERO {
                format!("{}", app.form_data.weight)
            } else {
                String::new()
            }
        }
        InputField::Inventory => {
            if app.form_data.inventory > Decimal::ZERO {
                format!("{}", app.form_data.inventory)
            } else {
                String::new()
            }
        }
        InputField::Receivables => {
            if app.form_data.receivables > Decimal::ZERO {
                format!("{}", app.form_data.receivables)
            } else {
                String::new()
            }
        }
        InputField::Liabilities => {
            if app.form_data.liabilities > Decimal::ZERO {
                format!("{}", app.form_data.liabilities)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

/// Render a single form field row
fn render_form_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    unit: &str,
    is_active: bool,
    input_value: Option<&str>,
    t: &crate::tui::theme::Theme,
) {
    let label_style = if is_active {
        Style::default().fg(t.gold).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(t.text_muted)
    };

    let value_style = if is_active {
        Style::default().fg(t.gold).add_modifier(Modifier::BOLD)
    } else if !value.is_empty() {
        Style::default().fg(t.text_primary)
    } else {
        Style::default().fg(t.text_muted).add_modifier(Modifier::ITALIC)
    };

    let display_value = if let Some(input) = input_value {
        format!("{}▏", input) // Show cursor for active input
    } else if value.is_empty() {
        "(empty)".to_string()
    } else if !unit.is_empty() && unit == "$" {
        format!("${}", value)
    } else if !unit.is_empty() {
        format!("{} {}", value, unit)
    } else {
        value.to_string()
    };

    let indicator = if is_active { "▶ " } else { "  " };
    let indicator_style = if is_active {
        Style::default().fg(t.gold)
    } else {
        Style::default()
    };

    let line = Line::from(vec![
        Span::styled(indicator, indicator_style),
        Span::styled(format!("{:<16}", format!("{}:", label)), label_style),
        Span::styled(display_value, value_style),
    ]);

    frame.render_widget(Paragraph::new(line), area);

    // Set cursor position if active
    if is_active && input_value.is_some() {
        let cursor_x = area.x + 2 + 16 + input_value.map(|s| s.len()).unwrap_or(0) as u16;
        frame.set_cursor_position((cursor_x, area.y));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RESULTS REPORT
// ═══════════════════════════════════════════════════════════════════════════

fn render_results_report(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let Some(results) = &app.results else {
        return;
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(t.emerald))
        .title(" Zakat Calculation Report ")
        .title_alignment(Alignment::Center)
        .title_style(t.title())
        .style(t.bg());

    frame.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5), // Summary Header
            Constraint::Length(1), // Divider
            Constraint::Min(0),    // Details Table
        ])
        .split(inner);

    // Summary Header - Two stat cards
    let summary_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    let total_style = if results.total_zakat_due > Decimal::ZERO {
        t.emerald
    } else {
        t.text_muted
    };

    StatCard::new("ZAKAT DUE", &format!("${:.2}", results.total_zakat_due))
        .value_color(total_style)
        .subtitle("2.5% of zakatable wealth")
        .render(frame, summary_layout[0]);

    StatCard::new("TOTAL WEALTH", &format!("${:.2}", results.total_assets))
        .value_color(t.text_primary)
        .subtitle("All tracked assets")
        .render(frame, summary_layout[1]);

    // Details Table
    let result_items = results.results();
    let rows: Vec<Row> = result_items
        .into_iter()
        .map(|res| match res {
            PortfolioItemResult::Success { details, .. } => {
                let color = if details.is_payable { t.emerald } else { t.text_muted };
                let status = if details.is_payable { "PAYABLE" } else { "EXEMPT" };

                Row::new(vec![
                    Cell::from(details.label.clone().unwrap_or_else(|| "Unknown".to_string())),
                    Cell::from(status).style(Style::default().fg(color)),
                    Cell::from(format!("${:.2}", details.zakat_due))
                        .style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ])
                .bottom_margin(1)
            }
            PortfolioItemResult::Failure { source, .. } => Row::new(vec![
                Cell::from(source),
                Cell::from("ERROR").style(Style::default().fg(t.error)),
                Cell::from("-"),
            ]),
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec!["ASSET", "STATUS", "DUE"])
            .style(Style::default().fg(t.text_muted).add_modifier(Modifier::UNDERLINED)),
    )
    .block(
        Block::default()
            .title(" Asset Breakdown ")
            .title_style(Style::default().fg(t.text_muted))
            .padding(Padding::new(1, 1, 1, 0)),
    );

    frame.render_widget(table, chunks[2]);
}

// ═══════════════════════════════════════════════════════════════════════════
// INPUT POPUP
// ═══════════════════════════════════════════════════════════════════════════

fn render_input_popup(frame: &mut Frame, app: &App) {
    let t = theme();

    let area = centered_rect(50, 20, frame.area());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(t.accent))
        .border_type(BorderType::Rounded)
        .title(" Input Required ")
        .title_style(t.accent_style())
        .style(t.bg());

    frame.render_widget(Clear, area);
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let label = match app.input_field {
        InputField::Label => "Enter Label:",
        InputField::Amount => "Enter Amount ($):",
        InputField::Weight => "Enter Weight (grams):",
        InputField::Inventory => "Enter Inventory Value ($):",
        InputField::Receivables => "Enter Receivables ($):",
        InputField::Liabilities => "Enter Liabilities ($):",
        InputField::Filename => "Enter Filename:",
        InputField::None => "Value:",
    };

    frame.render_widget(
        Paragraph::new(label).style(Style::default().fg(t.text_muted)),
        chunks[0],
    );

    // Input value with cursor indicator
    let input_value = app.input.value();
    let input_display = format!("{}_", input_value);

    frame.render_widget(
        Paragraph::new(input_display).style(Style::default().fg(t.gold).add_modifier(Modifier::BOLD)),
        chunks[1],
    );

    // Show cursor
    frame.set_cursor_position((
        chunks[1].x + app.input.visual_cursor() as u16,
        chunks[1].y,
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// HELP OVERLAY
// ═══════════════════════════════════════════════════════════════════════════

fn render_help(frame: &mut Frame, area: Rect) {
    let t = theme();

    let popup_area = centered_rect(65, 70, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Help ")
        .title_alignment(Alignment::Center)
        .title_style(t.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(t.border_active())
        .style(t.bg());

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("NAVIGATION", Style::default().fg(t.gold).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑ / k      ", t.accent_style()),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓ / j      ", t.accent_style()),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter      ", t.accent_style()),
            Span::raw("Confirm selection"),
        ]),
        Line::from(vec![
            Span::styled("  Esc        ", t.accent_style()),
            Span::raw("Cancel / Go back"),
        ]),
        Line::from(""),
        Line::from(Span::styled("QUICK ACTIONS", Style::default().fg(t.gold).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?          ", t.accent_style()),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q          ", t.accent_style()),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  d/Delete   ", t.accent_style()),
            Span::raw("Delete selected asset"),
        ]),
        Line::from(""),
        Line::from(Span::styled("SUPPORTED ASSETS", Style::default().fg(t.gold).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {} ", icons::BUILDING), Style::default()),
            Span::raw("Business - Cash, Inventory, Receivables"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} ", icons::GOLD), Style::default()),
            Span::raw("Gold/Silver - Weight-based with purity"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} ", icons::CHART), Style::default()),
            Span::raw("Investments - Stocks, Crypto (30% proxy)"),
        ]),
        Line::from(vec![
            Span::styled(format!("  {} ", icons::GRAIN), Style::default()),
            Span::raw("Agriculture - Harvest weight & irrigation"),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press [Esc] to close", Style::default().fg(t.text_muted).add_modifier(Modifier::ITALIC))),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, popup_area);
}

// ═══════════════════════════════════════════════════════════════════════════
// EDIT OVERLAY
// ═══════════════════════════════════════════════════════════════════════════

fn render_edit_overlay(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let items = app.portfolio.get_items();
    if items.is_empty() {
        return;
    }

    let popup_area = centered_rect(60, 60, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(t.border_active())
        .title(" Edit Assets ")
        .title_alignment(Alignment::Center)
        .title_style(t.title())
        .style(t.bg());

    frame.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    // Build list of assets
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let label = CalculateZakat::get_label(item).unwrap_or_else(|| format!("Item #{}", i + 1));
            let (icon, _type_color) = get_asset_icon_and_color(item);
            let is_selected = app.asset_index == i;

            let style = if is_selected {
                t.highlight()
            } else {
                Style::default().fg(t.text_primary)
            };

            let prefix = if is_selected { icons::ARROW_RIGHT } else { " " };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", prefix), style),
                Span::styled(format!("{} ", icon), style),
                Span::styled(label, style),
            ]))
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(inner);

    let list = List::new(list_items);
    frame.render_widget(list, chunks[0]);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(vec![
        Span::styled("[Enter] ", t.accent_style()),
        Span::raw("Edit  "),
        Span::styled("[d/Del] ", t.accent_style()),
        Span::raw("Delete  "),
        Span::styled("[Esc] ", t.accent_style()),
        Span::raw("Cancel"),
    ])])
    .alignment(Alignment::Center);

    frame.render_widget(instructions, chunks[1]);
}

// ═══════════════════════════════════════════════════════════════════════════
// STATUS BAR
// ═══════════════════════════════════════════════════════════════════════════

fn render_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let t = theme();

    let mode = match app.screen {
        Screen::Main => "DASHBOARD",
        Screen::AddAsset(AssetTypeSelection::Menu) => "SELECT ASSET",
        Screen::AddAsset(_) => "ADD ASSET",
        Screen::EditAsset(_) => "EDIT ASSET",
        Screen::Results => "REPORT",
        Screen::Help => "HELP",
        Screen::Loading => "LOADING",
    };

    // Status badge
    let status = if let Some((msg, kind)) = &app.message {
        let color = match kind {
            MessageType::Error => t.error,
            MessageType::Success => t.success,
            MessageType::Warning => t.warning,
            MessageType::Info => t.accent,
        };
        Span::styled(format!(" {} ", msg), Style::default().bg(color).fg(t.slate))
    } else {
        Span::styled(
            format!(" {} ", mode),
            Style::default().bg(t.slate_light).fg(t.text_muted),
        )
    };

    // Keys hint
    let keys = Span::styled(
        " [↑↓] Navigate  [Enter] Select  [?] Help  [Q] Quit ",
        Style::default().fg(t.text_muted),
    );

    let bar = Line::from(vec![status, Span::raw(" "), keys]);

    frame.render_widget(
        Paragraph::new(bar).style(t.bg()),
        area,
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// UTILITIES
// ═══════════════════════════════════════════════════════════════════════════

/// Helper to center a rect within a parent.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod ui_tests;
