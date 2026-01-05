//! # TUI Module
//!
//! Full-screen terminal user interface for the Zakat Calculator.
//!
//! This module provides a modern, interactive TUI built with ratatui.
//! Features a premium Islamic Finance aesthetic with Gold/Emerald themes.

pub mod app;
pub mod components;
pub mod event;
pub mod theme;
pub mod ui;

pub use app::App;
pub use event::handle_events;
pub use ui::ui;
