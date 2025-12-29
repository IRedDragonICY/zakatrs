pub mod config;
pub mod fitrah;
pub mod maal;
pub mod portfolio;
pub mod prelude;
pub mod traits;
pub mod types;
pub mod utils;

pub use config::{ZakatConfig, Madhab, NisabStandard};
pub use traits::CalculateZakat;
pub use types::{ZakatDetails, ZakatError, WealthType};
pub use portfolio::ZakatPortfolio;
