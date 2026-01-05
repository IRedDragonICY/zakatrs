//! Loading Spinner Widget
//!
//! Animated spinner for loading states.

#![allow(dead_code)]

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Gauge, Paragraph},
    Frame,
};

use super::super::theme::{icons, theme};

/// A loading spinner with optional message and progress.
pub struct LoadingSpinner<'a> {
    /// Message to display below spinner
    message: &'a str,
    /// Optional progress percentage (0-100)
    progress: Option<u16>,
    /// Frame index for animation
    frame: usize,
}

impl<'a> LoadingSpinner<'a> {
    /// Create a new loading spinner.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            progress: None,
            frame: 0,
        }
    }

    /// Set the animation frame (0-3).
    pub fn frame(mut self, frame: usize) -> Self {
        self.frame = frame % icons::SPINNER.len();
        self
    }

    /// Set progress percentage.
    pub fn progress(mut self, progress: u16) -> Self {
        self.progress = Some(progress.min(100));
        self
    }

    /// Render the loading spinner.
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let t = theme();

        // Center the content
        let v_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Length(1), // Spinner
                Constraint::Length(1), // Space
                Constraint::Length(1), // Message
                Constraint::Length(1), // Space
                Constraint::Length(1), // Progress (optional)
                Constraint::Percentage(35),
            ])
            .split(area);

        // Spinner character
        let spinner_char = icons::SPINNER[self.frame];
        let spinner = Paragraph::new(spinner_char)
            .style(Style::default().fg(t.gold).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(spinner, v_layout[1]);

        // Message
        let message = Paragraph::new(self.message)
            .style(t.subtitle())
            .alignment(Alignment::Center);
        frame.render_widget(message, v_layout[3]);

        // Progress bar (if provided)
        if let Some(pct) = self.progress {
            let progress_bar = Gauge::default()
                .gauge_style(Style::default().fg(t.gold).bg(t.slate_light))
                .percent(pct)
                .label(format!("{}%", pct));

            // Center the progress bar horizontally
            let h_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(25),
                    Constraint::Percentage(50),
                    Constraint::Percentage(25),
                ])
                .split(v_layout[5]);

            frame.render_widget(progress_bar, h_layout[1]);
        }
    }
}

/// Simple progress bar component.
pub struct ProgressBar {
    /// Current value
    value: u16,
    /// Maximum value
    max: u16,
    /// Width in characters
    width: u16,
}

impl ProgressBar {
    pub fn new(value: u16, max: u16) -> Self {
        Self {
            value,
            max,
            width: 20,
        }
    }

    pub fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    /// Generate the progress bar string.
    pub fn to_string(&self) -> String {
        let _t = theme();
        let pct = if self.max > 0 {
            (self.value as f32 / self.max as f32).min(1.0)
        } else {
            0.0
        };
        let filled = (pct * self.width as f32) as usize;
        let empty = self.width as usize - filled;

        format!(
            "{}{}",
            icons::PROGRESS_FULL.repeat(filled),
            icons::PROGRESS_EMPTY.repeat(empty)
        )
    }

    /// Render as a Line with styling.
    pub fn to_line(&self) -> Line<'static> {
        let t = theme();
        let pct = if self.max > 0 {
            (self.value as f32 / self.max as f32).min(1.0)
        } else {
            0.0
        };
        let filled = (pct * self.width as f32) as usize;
        let empty = self.width as usize - filled;

        Line::from(vec![
            Span::styled(
                icons::PROGRESS_FULL.repeat(filled),
                Style::default().fg(t.gold),
            ),
            Span::styled(
                icons::PROGRESS_EMPTY.repeat(empty),
                Style::default().fg(t.slate_light),
            ),
        ])
    }
}

/// Step indicator for wizard flows.
pub struct StepIndicator {
    /// Total number of steps
    total: usize,
    /// Current step (0-indexed)
    current: usize,
    /// Step labels
    labels: Vec<&'static str>,
}

impl StepIndicator {
    pub fn new(labels: Vec<&'static str>, current: usize) -> Self {
        let total = labels.len();
        Self {
            total,
            current: current.min(total.saturating_sub(1)),
            labels,
        }
    }

    /// Render the step indicator as a Line.
    pub fn to_line(&self) -> Line<'static> {
        let t = theme();
        let mut spans = Vec::new();

        for (i, label) in self.labels.iter().enumerate() {
            // Step indicator
            let (icon, style) = if i < self.current {
                (icons::STEP_COMPLETE, Style::default().fg(t.success))
            } else if i == self.current {
                (icons::STEP_CURRENT, Style::default().fg(t.gold).add_modifier(Modifier::BOLD))
            } else {
                (icons::STEP_PENDING, Style::default().fg(t.text_muted))
            };

            spans.push(Span::styled(format!(" {} ", icon), style));
            spans.push(Span::styled(label.to_string(), style));

            // Connector line (except for last item)
            if i < self.total - 1 {
                spans.push(Span::styled(
                    format!(" {} ", icons::STEP_LINE),
                    Style::default().fg(t.slate_light),
                ));
            }
        }

        Line::from(spans)
    }
}
