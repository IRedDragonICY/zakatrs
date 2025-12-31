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
pub mod assets;
pub mod math;
pub mod i18n;

#[cfg(any(target_arch = "wasm32", feature = "wasm"))]
pub mod wasm;

#[cfg(feature = "python")]
pub mod python;

pub use config::ZakatConfig;
pub use traits::CalculateZakat;
pub use types::{ZakatDetails, ZakatError, WealthType, ZakatExplanation};
pub use portfolio::ZakatPortfolio;
pub use assets::PortfolioItem;
pub use pricing::{Prices, StaticPriceProvider};
pub use madhab::{ZakatStrategy, ZakatRules};
pub use inputs::{IntoZakatDecimal, InputLocale, LocalizedInput, with_locale};
#[cfg(feature = "async")]
pub use pricing::PriceProvider;

