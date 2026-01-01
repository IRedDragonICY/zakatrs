use std::sync::Arc;
use crate::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;

// --- Facade: Configuration ---
#[derive(uniffi::Record)]
pub struct KotlinZakatConfig {
    pub gold_price: f64,
    pub silver_price: f64,
}

#[derive(uniffi::Object)]
pub struct KotlinConfigWrapper {
    pub inner: ZakatConfig,
}

#[uniffi::export]
impl KotlinConfigWrapper {
    #[uniffi::constructor]
    pub fn new(gold_price: f64, silver_price: f64) -> Arc<Self> {
        let mut cfg = ZakatConfig::default();
        cfg.gold_price_per_gram = Decimal::from_f64_retain(gold_price).unwrap_or(dec!(0.0));
        cfg.silver_price_per_gram = Decimal::from_f64_retain(silver_price).unwrap_or(dec!(0.0));
        Arc::new(Self { inner: cfg })
    }
}

// --- Facade: Assets (Business) ---
#[derive(uniffi::Object)]
pub struct KotlinBusinessZakat {
    pub cash: f64,
    pub merchandise: f64,
    pub receivables: f64,
    pub debt: f64,
    pub expenses: f64,
}

#[uniffi::export]
impl KotlinBusinessZakat {
    #[uniffi::constructor]
    pub fn new(cash: f64, merchandise: f64, receivables: f64, debt: f64, expenses: f64) -> Arc<Self> {
        Arc::new(Self {
            cash,
            merchandise,
            receivables,
            debt,
            expenses,
        })
    }

    pub fn calculate(&self, config: Arc<KotlinConfigWrapper>) -> f64 {
        let business = BusinessZakat::new()
            .cash(Decimal::from_f64_retain(self.cash).unwrap_or(Decimal::ZERO))
            .inventory(Decimal::from_f64_retain(self.merchandise).unwrap_or(Decimal::ZERO))
            .receivables(Decimal::from_f64_retain(self.receivables).unwrap_or(Decimal::ZERO))
            .debt(Decimal::from_f64_retain(self.debt).unwrap_or(Decimal::ZERO))
            .liabilities(Decimal::from_f64_retain(self.expenses).unwrap_or(Decimal::ZERO));
            
        match business.calculate_zakat(&config.inner) {
            Ok(result) => result.zakat_due.to_f64().unwrap_or(0.0),
            Err(_) => 0.0
        }
    }
}

// --- Facade: Assets (Gold) ---
#[derive(uniffi::Object)]
pub struct KotlinPreciousMetals {
    pub gold_grams: f64,
    pub silver_grams: f64,
}

#[uniffi::export]
impl KotlinPreciousMetals {
    #[uniffi::constructor]
    pub fn new(gold_grams: f64, silver_grams: f64) -> Arc<Self> {
        Arc::new(Self { gold_grams, silver_grams })
    }

    pub fn calculate(&self, config: Arc<KotlinConfigWrapper>) -> f64 {
        let mut total_zakat = 0.0;

        if self.gold_grams > 0.0 {
            let metals = PreciousMetals::new()
                .weight(Decimal::from_f64_retain(self.gold_grams).unwrap_or(Decimal::ZERO))
                .metal_type(WealthType::Gold);
            if let Ok(res) = metals.calculate_zakat(&config.inner) {
                total_zakat += res.zakat_due.to_f64().unwrap_or(0.0);
            }
        }

        if self.silver_grams > 0.0 {
            let metals = PreciousMetals::new()
                .weight(Decimal::from_f64_retain(self.silver_grams).unwrap_or(Decimal::ZERO))
                .metal_type(WealthType::Silver);
             if let Ok(res) = metals.calculate_zakat(&config.inner) {
                total_zakat += res.zakat_due.to_f64().unwrap_or(0.0);
            }
        }

        total_zakat
    }
}
