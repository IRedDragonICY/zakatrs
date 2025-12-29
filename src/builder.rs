use crate::types::ZakatError;

/// Trait for builders that produce a Zakat asset or configuration.
///
/// This creates a unified interface for object creation across the crate.
pub trait AssetBuilder<T> {
    /// Builds the final object, returning a Result.
    fn build(self) -> Result<T, ZakatError>;
}
