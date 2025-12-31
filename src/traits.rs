use crate::types::{ZakatDetails, ZakatError};
use crate::config::ZakatConfig;
use std::borrow::Cow;

/// Helper trait to allow flexible config arguments.
pub trait ZakatConfigArgument {
    fn resolve_config(&self) -> Cow<'_, ZakatConfig>;
}

// 1. Support passing &ZakatConfig directly
impl ZakatConfigArgument for &ZakatConfig {
    fn resolve_config(&self) -> Cow<'_, ZakatConfig> {
        Cow::Borrowed(self)
    }
}

// 2. Support passing () for default config
impl ZakatConfigArgument for () {
    fn resolve_config(&self) -> Cow<'_, ZakatConfig> {
        Cow::Owned(ZakatConfig::default())
    }
}

// 3. Support passing Option<&ZakatConfig>
impl ZakatConfigArgument for Option<&ZakatConfig> {
    fn resolve_config(&self) -> Cow<'_, ZakatConfig> {
        match self {
            Some(c) => Cow::Borrowed(c),
            None => Cow::Owned(ZakatConfig::default()),
        }
    }
}

/// Trait to be implemented by all Zakat calculators.
pub trait CalculateZakat {
    /// Calculate Zakat details with flexible config argument.
    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError>;
    
    /// Convenience method using default config.
    fn calculate(&self) -> Result<ZakatDetails, ZakatError> {
        self.calculate_zakat(())
    }

    fn get_label(&self) -> Option<String> { None }
    fn get_id(&self) -> uuid::Uuid;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncCalculateZakat: Send + Sync {
    async fn calculate_zakat_async<C: ZakatConfigArgument + Send + Sync>(&self, config: C) -> Result<ZakatDetails, ZakatError>;
    fn get_label(&self) -> Option<String> { None }
    fn get_id(&self) -> uuid::Uuid;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<T> AsyncCalculateZakat for T 
where T: CalculateZakat + Sync + Send 
{
    async fn calculate_zakat_async<C: ZakatConfigArgument + Send + Sync>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        self.calculate_zakat(config)
    }
    fn get_label(&self) -> Option<String> { self.get_label() }
    fn get_id(&self) -> uuid::Uuid { self.get_id() }
}
