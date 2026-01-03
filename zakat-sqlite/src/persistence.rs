//! Persistence traits for Ledger Events.
//!
//! Defines the `LedgerStore` trait for saving and loading ledger events.

use async_trait::async_trait;
use std::path::PathBuf;
use zakat_core::types::{ZakatError, InvalidInputDetails};
use zakat_ledger::events::LedgerEvent;

/// Trait for persisting Zakat Ledger events.
#[async_trait]
pub trait LedgerStore: Send + Sync {
    /// Saves a single event to the store.
    async fn save_event(&self, event: &LedgerEvent) -> Result<(), ZakatError>;
    
    /// Loads all events from the store.
    async fn load_events(&self) -> Result<Vec<LedgerEvent>, ZakatError>;
}

/// A simple JSON file-based implementation of LedgerStore.
pub struct JsonFileStore {
    path: PathBuf,
}

impl JsonFileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait]
impl LedgerStore for JsonFileStore {
    async fn save_event(&self, event: &LedgerEvent) -> Result<(), ZakatError> {
        let mut events = self.load_events().await.unwrap_or_default();
        events.push(event.clone());
        
        let json = serde_json::to_string_pretty(&events).map_err(|e| ZakatError::InvalidInput(Box::new(InvalidInputDetails { 
            field: "ledger".to_string(), 
            value: "json".to_string(),
            reason_key: "error-parse-json".to_string(),
            args: Some(std::collections::HashMap::from([("error".to_string(), e.to_string())])),
            source_label: Some("JsonFileStore".to_string()),
            asset_id: None
        })))?;
        
        tokio::fs::write(&self.path, json).await.map_err(|e: std::io::Error| ZakatError::NetworkError(format!("IO Error: {}", e)))?;
        Ok(())
    }

    async fn load_events(&self) -> Result<Vec<LedgerEvent>, ZakatError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        
        let content = tokio::fs::read_to_string(&self.path).await.map_err(|e: std::io::Error| ZakatError::NetworkError(format!("IO Error: {}", e)))?;
        
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        serde_json::from_str(&content).map_err(|e| ZakatError::InvalidInput(Box::new(InvalidInputDetails {
            field: "ledger".to_string(),
            value: "json".to_string(),
            reason_key: "error-parse-json".to_string(),
            args: Some(std::collections::HashMap::from([("error".to_string(), e.to_string())])),
            source_label: Some("JsonFileStore".to_string()),
            asset_id: None
        })))
    }
}
