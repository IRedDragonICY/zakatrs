//! Stat Card Widget
//!
//! A reusable card component for displaying statistics with title/value pairs.

#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::super::theme::theme;

/// A stat card displaying a title and value with optional styling.
pub struct StatCard<'a> {
    /// Card title/label
    title: &'a str,
    /// Card value to display
    value: &'a str,
    /// Color for the value text
    value_color: Color,
    /// Whether this card is highlighted/focused
    highlighted: bool,
    /// Optional subtitle or additional info
    subtitle: Option<&'a str>,
}

impl<'a> StatCard<'a> {
    /// Create a new stat card with title and value.
    pub fn new(title: &'a str, value: &'a str) -> Self {
        Self {
            title,
            value,
            value_color: theme().text_primary,
            highlighted: false,
            subtitle: None,
        }
    }

    /// Set the value color.
    pub fn value_color(mut self, color: Color) -> Self {
        self.value_color = color;
        self
    }

    /// Set whether this card is highlighted.
    pub fn highlighted(mut self, highlighted: bool) -> Self {
        self.highlighted = highlighted;
        self
    }

    /// Set an optional subtitle.
    pub fn subtitle(mut self, subtitle: &'a str) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    /// Render the stat card to the frame.
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let t = theme();

        let border_style = if self.highlighted {
            t.border_active()
        } else {
            t.border_inactive()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(border_style)
            .style(t.bg());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate content layout
        let has_subtitle = self.subtitle.is_some();
        let content_constraints = if has_subtitle {
            vec![
                Constraint::Length(1), // Title
                Constraint::Length(1), // Value
                Constraint::Length(1), // Subtitle
            ]
        } else {
            vec![
                Constraint::Length(1), // Title
                Constraint::Min(1),    // Value (centered)
            ]
        };

        let content = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(content_constraints)
            .split(inner);

        // Title
        frame.render_widget(
            Paragraph::new(self.title)
                .style(t.subtitle())
                .alignment(Alignment::Left),
            content[0],
        );

        // Value
        frame.render_widget(
            Paragraph::new(self.value)
                .style(Style::default().fg(self.value_color).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Left),
            content[1],
        );

        // Subtitle (if any)
        if let Some(subtitle) = self.subtitle
            && content.len() > 2 {
                frame.render_widget(
                    Paragraph::new(subtitle)
                        .style(Style::default().fg(t.text_muted))
                        .alignment(Alignment::Left),
                    content[2],
                );
            }
    }
}

/// A compact inline stat display (label: value).
pub struct InlineStat<'a> {
    label: &'a str,
    value: &'a str,
    label_color: Color,
    value_color: Color,
}

impl<'a> InlineStat<'a> {
    pub fn new(label: &'a str, value: &'a str) -> Self {
        let t = theme();
        Self {
            label,
            value,
            label_color: t.text_muted,
            value_color: t.text_primary,
        }
    }

    pub fn label_color(mut self, color: Color) -> Self {
        self.label_color = color;
        self
    }

    pub fn value_color(mut self, color: Color) -> Self {
        self.value_color = color;
        self
    }

    pub fn to_line(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled(self.label, Style::default().fg(self.label_color)),
            Span::raw(" "),
            Span::styled(self.value, Style::default().fg(self.value_color).add_modifier(Modifier::BOLD)),
        ])
    }
}
