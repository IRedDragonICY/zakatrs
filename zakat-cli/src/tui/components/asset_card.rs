//! Asset Card Widget
//!
//! A selectable card component for asset type selection.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph},
    Frame,
};

use super::super::theme::theme;

/// An asset type selection card with icon, title, and description.
pub struct AssetCard<'a> {
    /// Icon to display (emoji)
    icon: &'a str,
    /// Card title
    title: &'a str,
    /// Description text
    description: &'a str,
    /// Whether this card is selected/focused
    selected: bool,
}

impl<'a> AssetCard<'a> {
    /// Create a new asset card.
    pub fn new(icon: &'a str, title: &'a str, description: &'a str) -> Self {
        Self {
            icon,
            title,
            description,
            selected: false,
        }
    }

    /// Set whether this card is selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Render the asset card to the frame.
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let t = theme();

        let border_style = if self.selected {
            Style::default().fg(t.gold).add_modifier(Modifier::BOLD)
        } else {
            t.border_inactive()
        };

        let card_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .style(t.bg());

        frame.render_widget(card_block.clone(), area);

        let inner_area = card_block.inner(area);

        // Layout: Icon | Separator | Content
        let card_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(4), // Icon width
                Constraint::Min(0),    // Content
            ])
            .split(inner_area);

        // Icon (centered vertically)
        let icon_v_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(card_chunks[0]);

        let icon_style = if self.selected {
            Style::default().fg(t.gold)
        } else {
            Style::default()
        };

        frame.render_widget(
            Paragraph::new(self.icon)
                .style(icon_style)
                .alignment(Alignment::Center),
            icon_v_center[1],
        );

        // Content with left border separator
        let sep_style = if self.selected {
            Style::default().fg(t.gold)
        } else {
            Style::default().fg(t.slate_light)
        };

        let content_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(sep_style)
            .padding(Padding::new(1, 1, 0, 0));

        let content_area = card_chunks[1];
        frame.render_widget(content_block.clone(), content_area);

        let text_inner_area = content_block.inner(content_area);

        // Title and description styles
        let title_style = if self.selected {
            Style::default().fg(t.gold).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(t.text_primary).add_modifier(Modifier::BOLD)
        };

        let text = vec![
            Line::from(Span::styled(self.title, title_style)),
            Line::from(Span::styled(self.description, Style::default().fg(t.text_muted))),
        ];

        // Center text vertically
        let v_center = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .split(text_inner_area);

        frame.render_widget(
            Paragraph::new(text),
            v_center[1],
        );
    }
}

/// Data for asset type options.
pub struct AssetTypeOption {
    pub icon: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

impl AssetTypeOption {
    pub const BUSINESS: Self = Self {
        icon: "🏢",
        title: "Business Assets",
        description: "Trade goods, cash, receivables",
    };

    pub const GOLD: Self = Self {
        icon: "⚱️",
        title: "Gold",
        description: "Jewelry, bars, stored wealth",
    };

    pub const SILVER: Self = Self {
        icon: "🥈",
        title: "Silver",
        description: "Utensils, coins, savings",
    };

    pub const CASH: Self = Self {
        icon: "💵",
        title: "Cash / Savings",
        description: "Bank accounts, cash on hand",
    };

    pub const INVESTMENT: Self = Self {
        icon: "📈",
        title: "Investments",
        description: "Stocks, Crypto, Mutual Funds",
    };

    pub const AGRICULTURE: Self = Self {
        icon: "🌾",
        title: "Agriculture",
        description: "Crops, harvest, produce",
    };

    pub const BACK: Self = Self {
        icon: "↩",
        title: "Back",
        description: "Return to main menu",
    };

    /// Returns all asset type options in order.
    pub fn all() -> [Self; 7] {
        [
            Self::BUSINESS,
            Self::GOLD,
            Self::SILVER,
            Self::CASH,
            Self::INVESTMENT,
            Self::AGRICULTURE,
            Self::BACK,
        ]
    }
}
