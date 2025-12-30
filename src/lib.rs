#[macro_use]
pub mod macros;
pub mod config;
pub mod fitrah;
pub mod inputs;
pub mod madhab;
pub mod maal;
pub mod portfolio;
pub mod prelude;
pub mod pricing;
pub mod traits;
pub mod types;
pub mod utils;

pub use config::ZakatConfig;
pub use traits::CalculateZakat;
pub use types::{ZakatDetails, ZakatError, WealthType};
pub use portfolio::ZakatPortfolio;
pub use pricing::{Prices, StaticPriceProvider};
pub use madhab::{ZakatStrategy, ZakatRules};
#[cfg(feature = "async")]
pub use pricing::PriceProvider;

