pub mod config;
pub mod fitrah;
pub mod maal;
pub mod portfolio;
pub mod traits;
pub mod types;

pub use config::ZakatConfig;
pub use traits::CalculateZakat;
pub use types::{ZakatDetails, ZakatError, WealthType};
pub use portfolio::ZakatPortfolio;
