
use crate::types::{ZakatDetails, ZakatError};

/// Trait to be implemented by all Zakat calculators.
pub trait CalculateZakat {
    /// Calculate Zakat details.
    ///
    /// * `Result<ZakatDetails, ZakatError>`
    fn calculate_zakat(&self, config: &crate::config::ZakatConfig) -> Result<ZakatDetails, ZakatError>;
    
    /// Returns the label of the asset, if any.
    fn get_label(&self) -> Option<String> {
        None
    }

    /// Returns the stable unique identifier for this asset.
    fn get_id(&self) -> uuid::Uuid;
}

/// Async version of the CalculateZakat trait.
///
/// This trait is automatically implemented for any type that implements `CalculateZakat + Send + Sync`.
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncCalculateZakat: Send + Sync {
    /// Calculate Zakat details asynchronously.
    async fn calculate_zakat_async(&self, config: &crate::config::ZakatConfig) -> Result<ZakatDetails, ZakatError>;
    
    /// Returns the label of the asset, if any.
    fn get_label(&self) -> Option<String> { None }

    /// Returns the stable unique identifier for this asset.
    fn get_id(&self) -> uuid::Uuid;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<T> AsyncCalculateZakat for T 
where T: CalculateZakat + Sync + Send 
{
    async fn calculate_zakat_async(&self, config: &crate::config::ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        self.calculate_zakat(config)
    }

    fn get_label(&self) -> Option<String> {
        self.get_label()
    }

    fn get_id(&self) -> uuid::Uuid {
        self.get_id()
    }
}
