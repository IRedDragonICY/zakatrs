pub mod config;
pub mod fitrah;
pub mod inputs;
pub mod madhab;
pub mod maal;
pub mod portfolio;
pub mod prelude;
pub mod traits;
pub mod types;
pub mod utils;

pub use config::ZakatConfig;
pub use traits::CalculateZakat;
pub use types::{ZakatDetails, ZakatError, WealthType};
pub use portfolio::ZakatPortfolio;
