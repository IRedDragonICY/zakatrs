//! Premium Islamic Finance Theme System
//!
//! A centralized theme providing Gold/Emerald color palette with dark slate background.
//! Designed for a premium, modern terminal user interface.

use ratatui::style::{Color, Modifier, Style};

/// The main theme struct containing all colors and pre-computed styles.
#[derive(Debug, Clone)]
pub struct Theme {
    // Primary brand colors
    /// Metallic gold - primary accent color
    pub gold: Color,
    /// Islamic emerald green - success and positive values
    pub emerald: Color,
    /// Dark slate - main background
    pub slate: Color,
    /// Light slate - panel/card backgrounds
    pub slate_light: Color,

    // Semantic colors
    /// Primary text color (near-white)
    pub text_primary: Color,
    /// Muted/secondary text color
    pub text_muted: Color,
    /// Error color (red)
    pub error: Color,
    /// Warning color (gold)
    pub warning: Color,
    /// Success color (emerald)
    pub success: Color,
    /// Accent color (cyan)
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            gold: Color::Rgb(212, 175, 55),
            emerald: Color::Rgb(16, 185, 129),
            slate: Color::Rgb(15, 23, 42),
            slate_light: Color::Rgb(30, 41, 59),
            text_primary: Color::Rgb(248, 250, 252),
            text_muted: Color::Rgb(148, 163, 184),
            error: Color::Rgb(239, 68, 68),
            warning: Color::Rgb(212, 175, 55),
            success: Color::Rgb(16, 185, 129),
            accent: Color::Cyan,
        }
    }
}

#[allow(dead_code)]
impl Theme {
    /// Creates a new theme with default colors.
    pub fn new() -> Self {
        Self::default()
    }

    // ─────────────────────────────────────────────────────────────
    // Pre-computed Styles
    // ─────────────────────────────────────────────────────────────

    /// Title style - bold gold text
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.gold)
            .add_modifier(Modifier::BOLD)
    }

    /// Subtitle/label style - muted text
    pub fn subtitle(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    /// Primary text style
    pub fn text(&self) -> Style {
        Style::default().fg(self.text_primary)
    }

    /// Highlighted/selected item style
    pub fn highlight(&self) -> Style {
        Style::default()
            .fg(self.slate)
            .bg(self.gold)
            .add_modifier(Modifier::BOLD)
    }

    /// Active border style
    pub fn border_active(&self) -> Style {
        Style::default().fg(self.gold)
    }

    /// Inactive border style
    pub fn border_inactive(&self) -> Style {
        Style::default().fg(self.slate_light)
    }

    /// Success style - emerald text
    pub fn success_style(&self) -> Style {
        Style::default()
            .fg(self.success)
            .add_modifier(Modifier::BOLD)
    }

    /// Error style - red text
    pub fn error_style(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    /// Warning style - gold text
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Accent style - cyan text
    pub fn accent_style(&self) -> Style {
        Style::default().fg(self.accent)
    }

    /// Background style for main area
    pub fn bg(&self) -> Style {
        Style::default().bg(self.slate)
    }

    /// Background style for panels/cards
    pub fn bg_panel(&self) -> Style {
        Style::default().bg(self.slate_light)
    }

    /// Value display style - bold primary text
    pub fn value(&self) -> Style {
        Style::default()
            .fg(self.text_primary)
            .add_modifier(Modifier::BOLD)
    }

    /// Positive value style - bold emerald
    pub fn value_positive(&self) -> Style {
        Style::default()
            .fg(self.emerald)
            .add_modifier(Modifier::BOLD)
    }

    /// Negative value style - bold red
    pub fn value_negative(&self) -> Style {
        Style::default()
            .fg(self.error)
            .add_modifier(Modifier::BOLD)
    }

    // ─────────────────────────────────────────────────────────────
    // Asset Type Colors
    // ─────────────────────────────────────────────────────────────

    /// Color for business assets
    pub fn asset_business(&self) -> Color {
        self.accent
    }

    /// Color for gold assets
    pub fn asset_gold(&self) -> Color {
        self.gold
    }

    /// Color for silver assets
    pub fn asset_silver(&self) -> Color {
        Color::Rgb(192, 192, 192)
    }

    /// Color for cash/savings
    pub fn asset_cash(&self) -> Color {
        self.accent
    }

    /// Color for investments
    pub fn asset_investment(&self) -> Color {
        self.emerald
    }

    /// Color for agriculture
    pub fn asset_agriculture(&self) -> Color {
        Color::Rgb(34, 197, 94)
    }

    /// Color for livestock
    pub fn asset_livestock(&self) -> Color {
        self.warning
    }
}

/// Global theme instance for convenience.
/// In a more complex app, this could be configurable.
pub static THEME: std::sync::LazyLock<Theme> = std::sync::LazyLock::new(Theme::default);

/// Convenience function to get the default theme.
pub fn theme() -> &'static Theme {
    &THEME
}

// ─────────────────────────────────────────────────────────────────────
// Unicode Icons
// ─────────────────────────────────────────────────────────────────────

/// Icons used throughout the TUI
#[allow(dead_code)]
pub mod icons {
    pub const MOON: &str = "🌙";
    pub const BUILDING: &str = "🏢";
    pub const GOLD: &str = "🪙";
    pub const SILVER: &str = "🥈";
    pub const CASH: &str = "💵";
    pub const CHART: &str = "📈";
    pub const GRAIN: &str = "🌾";
    pub const LIVESTOCK: &str = "🐄";
    pub const PACKAGE: &str = "📦";
    pub const SAVE: &str = "💾";
    pub const FOLDER: &str = "📂";
    pub const CALCULATOR: &str = "🧮";
    pub const HELP: &str = "❓";
    pub const CLOSE: &str = "✕";
    pub const BACK: &str = "↩";
    pub const CHECK: &str = "✓";
    pub const CROSS: &str = "✗";
    pub const EDIT: &str = "✎";
    pub const ADD: &str = "+";
    pub const BULLET: &str = "•";
    pub const ARROW_RIGHT: &str = "➜";
    pub const SEPARATOR: &str = "│";
    
    // Spinner frames for loading animation
    pub const SPINNER: &[&str] = &["◐", "◓", "◑", "◒"];
    
    // Progress bar characters
    pub const PROGRESS_FULL: &str = "█";
    pub const PROGRESS_EMPTY: &str = "░";
    pub const PROGRESS_HALF: &str = "▓";

    // Step indicator
    pub const STEP_COMPLETE: &str = "●";
    pub const STEP_CURRENT: &str = "◉";
    pub const STEP_PENDING: &str = "○";
    pub const STEP_LINE: &str = "───";
}
