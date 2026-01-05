//! Persistent CLI Configuration Loader
//!
//! This module provides platform-aware configuration file loading for the Zakat CLI.
//! Configuration is loaded from `~/.config/zakat/config.toml` on Linux/macOS
//! or `%APPDATA%\zakat\config.toml` on Windows.

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::path::PathBuf;
use tracing::{debug, warn};

/// CLI Configuration structure loaded from TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct CliConfig {
    /// Default gold price per gram.
    pub gold_price: Option<Decimal>,
    /// Default silver price per gram.
    pub silver_price: Option<Decimal>,
    /// Default locale code (e.g., "en-US", "ar-SA").
    pub locale: Option<String>,
    /// Default currency code (e.g., "USD", "SAR").
    pub currency: Option<String>,
    /// Preferred Madhab ("hanafi", "shafi", "hanbali", "maliki").
    pub madhab: Option<String>,
    /// Nisab standard ("gold", "silver", "lower-of-two").
    pub nisab_standard: Option<String>,
    /// Enable file logging by default.
    pub enable_logging: Option<bool>,
    /// Offline mode by default.
    pub offline: Option<bool>,
}

#[allow(dead_code)]
impl CliConfig {
    /// Returns the platform-specific configuration directory.
    /// - Linux: ~/.config/zakat/
    /// - macOS: ~/Library/Application Support/zakat/
    /// - Windows: %APPDATA%\zakat\
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("zakat"))
    }

    /// Returns the full path to the config file.
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("config.toml"))
    }

    /// Attempts to load configuration from the default config file location.
    /// Returns `CliConfig::default()` if the file doesn't exist or fails to parse.
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            debug!("Could not determine config directory");
            return Self::default();
        };

        if !path.exists() {
            debug!("No config file found at {:?}", path);
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => {
                match toml::from_str::<CliConfig>(&content) {
                    Ok(config) => {
                        debug!("Loaded configuration from {:?}", path);
                        config
                    }
                    Err(e) => {
                        warn!("Failed to parse config file {:?}: {}", path, e);
                        Self::default()
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read config file {:?}: {}", path, e);
                Self::default()
            }
        }
    }

    /// Saves the current configuration to the default config file location.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory"
            ))?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        std::fs::write(&path, content)?;
        debug!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Creates a sample configuration file at the default location.
    pub fn create_sample() -> Result<PathBuf, std::io::Error> {
        let sample = CliConfig {
            gold_price: Some(Decimal::from(85)),
            silver_price: Some(Decimal::from(1)),
            locale: Some("en-US".to_string()),
            currency: Some("USD".to_string()),
            madhab: Some("hanafi".to_string()),
            nisab_standard: Some("gold".to_string()),
            enable_logging: Some(false),
            offline: Some(false),
        };
        sample.save()?;
        Ok(Self::config_path().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = CliConfig::default();
        assert!(config.gold_price.is_none());
        assert!(config.locale.is_none());
    }

    #[test]  
    fn test_config_serialization() {
        let config = CliConfig {
            gold_price: Some(Decimal::from(85)),
            currency: Some("USD".to_string()),
            ..Default::default()
        };
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("gold-price"));
        assert!(toml_str.contains("USD"));
    }
}
