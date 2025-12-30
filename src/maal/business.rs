use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::config::ZakatConfig;
use crate::inputs::IntoZakatDecimal;
use crate::builder::AssetBuilder;

/// Represents Business Assets for Zakat Calculation.
/// 
/// This struct unifies the assets data and calculation context (liabilities, hawl, etc.).
/// Use `BusinessZakat::builder()` to construct.
#[derive(Debug, Clone, PartialEq)]
pub struct BusinessZakat {
    // Assets
    pub cash_on_hand: Decimal,
    pub inventory_value: Decimal,
    pub receivables: Decimal,
    // Liabilities
    pub short_term_liabilities: Decimal, // Liabilities related to business operations
    pub liabilities_due_now: Decimal,    // Additional debts/liabilities due now (deductible)
    // Context
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

impl BusinessZakat {
    /// Returns a new builder for creating `BusinessZakat`.
    pub fn builder() -> BusinessZakatBuilder {
        BusinessZakatBuilder::default()
    }
}

#[derive(Default)]
pub struct BusinessZakatBuilder {
    cash_on_hand: Option<Decimal>,
    inventory_value: Option<Decimal>,
    receivables: Option<Decimal>,
    short_term_liabilities: Option<Decimal>,
    liabilities_due_now: Option<Decimal>,
    hawl_satisfied: Option<bool>,
    label: Option<String>,
}

impl BusinessZakatBuilder {
    pub fn cash(mut self, cash: impl IntoZakatDecimal) -> Self {
        if let Ok(val) = cash.into_zakat_decimal() {
             self.cash_on_hand = Some(val);
        }
        self
    }

    pub fn inventory(mut self, inventory: impl IntoZakatDecimal) -> Self {
        if let Ok(val) = inventory.into_zakat_decimal() {
            self.inventory_value = Some(val);
        }
        self
    }

    pub fn receivables(mut self, receivables: impl IntoZakatDecimal) -> Self {
        if let Ok(val) = receivables.into_zakat_decimal() {
            self.receivables = Some(val);
        }
        self
    }

    /// Sets short-term business liabilities (deducted from gross asstes).
    pub fn liabilities(mut self, liabilities: impl IntoZakatDecimal) -> Self {
        if let Ok(val) = liabilities.into_zakat_decimal() {
            self.short_term_liabilities = Some(val);
        }
        self
    }

    /// Sets additional liabilities due immediately (deducted from total).
    pub fn debt(mut self, debt: impl IntoZakatDecimal) -> Self {
        if let Ok(val) = debt.into_zakat_decimal() {
            self.liabilities_due_now = Some(val);
        }
        self
    }

    pub fn hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = Some(satisfied);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

use crate::builder::Validate;

impl Validate for BusinessZakatBuilder {
    fn validate(&self) -> Result<(), ZakatError> {
        let cash = self.cash_on_hand.unwrap_or(Decimal::ZERO);
        let inventory = self.inventory_value.unwrap_or(Decimal::ZERO);
        let receivables = self.receivables.unwrap_or(Decimal::ZERO);
        let liabilities = self.short_term_liabilities.unwrap_or(Decimal::ZERO);
        let liabilities_due_now = self.liabilities_due_now.unwrap_or(Decimal::ZERO);

        if cash < Decimal::ZERO || inventory < Decimal::ZERO || receivables < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Business assets must be non-negative".to_string(), self.label.clone()));
        }
        if liabilities < Decimal::ZERO || liabilities_due_now < Decimal::ZERO {
             return Err(ZakatError::InvalidInput("Liabilities must be non-negative".to_string(), self.label.clone()));
        }
        Ok(())
    }
}

impl AssetBuilder<BusinessZakat> for BusinessZakatBuilder {
    fn build(self) -> Result<BusinessZakat, ZakatError> {
        self.validate()?;
        
        let cash = self.cash_on_hand.unwrap_or(Decimal::ZERO);
        let inventory = self.inventory_value.unwrap_or(Decimal::ZERO);
        let receivables = self.receivables.unwrap_or(Decimal::ZERO);
        let liabilities = self.short_term_liabilities.unwrap_or(Decimal::ZERO);
        let liabilities_due_now = self.liabilities_due_now.unwrap_or(Decimal::ZERO);

        Ok(BusinessZakat {
            cash_on_hand: cash,
            inventory_value: inventory,
            receivables,
            short_term_liabilities: liabilities,
            liabilities_due_now,
            hawl_satisfied: self.hawl_satisfied.unwrap_or(true), // Default to true if not specified
            label: self.label,
        })
    }
}

impl CalculateZakat for BusinessZakat {
    fn calculate_zakat(&self, config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // For LowerOfTwo or Silver standard, we need silver price too
        let needs_silver = matches!(
            config.cash_nisab_standard,
            crate::madhab::NisabStandard::Silver | crate::madhab::NisabStandard::LowerOfTwo
        );
        
        if config.gold_price_per_gram <= Decimal::ZERO && !needs_silver {
            return Err(ZakatError::ConfigurationError("Gold price needed for Business Nisab".to_string(), self.label.clone()));
        }
        if needs_silver && config.silver_price_per_gram <= Decimal::ZERO {
            return Err(ZakatError::ConfigurationError("Silver price needed for Business Nisab with current standard".to_string(), self.label.clone()));
        }
        
        // Dynamic Nisab threshold based on config (Gold, Silver, or LowerOfTwo)
        let nisab_threshold_value = config.get_monetary_nisab_threshold();

        if !self.hawl_satisfied {
            return Ok(ZakatDetails::below_threshold(nisab_threshold_value, crate::types::WealthType::Business, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        
        let gross_assets = self.cash_on_hand
            .checked_add(self.inventory_value)
            .and_then(|v| v.checked_add(self.receivables))
            .ok_or(ZakatError::CalculationError("Overflow summing business assets".to_string(), self.label.clone()))?;
            
        let total_liabilities = self.short_term_liabilities
            .checked_add(self.liabilities_due_now)
            .ok_or(ZakatError::CalculationError("Overflow summing business liabilities".to_string(), self.label.clone()))?;

        // Zakat Rate is 2.5%
        let rate = dec!(0.025);

        // Build calculation trace
        let net_assets = gross_assets - total_liabilities;
        let mut trace = vec![
            crate::types::CalculationStep::initial("Cash on Hand", self.cash_on_hand),
            crate::types::CalculationStep::add("Inventory Value", self.inventory_value),
            crate::types::CalculationStep::add("Receivables", self.receivables),
            crate::types::CalculationStep::result("Gross Assets", gross_assets),
            crate::types::CalculationStep::subtract("Short-term Liabilities", self.short_term_liabilities),
            crate::types::CalculationStep::subtract("Debts Due Now", self.liabilities_due_now),
            crate::types::CalculationStep::result("Net Business Assets", net_assets),
            crate::types::CalculationStep::compare("Nisab Threshold", nisab_threshold_value),
        ];

        // We rely on ZakatDetails::with_trace to calculate final amounts, 
        // but we add a trace step for rate/info.
        if net_assets >= nisab_threshold_value && net_assets > Decimal::ZERO {
            trace.push(crate::types::CalculationStep::rate("Applied Rate (2.5%)", rate));
        } else {
             trace.push(crate::types::CalculationStep::info("Net Assets below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails::with_trace(
            gross_assets, 
            total_liabilities, 
            nisab_threshold_value, 
            rate, 
            crate::types::WealthType::Business, 
            trace
        ).with_label(self.label.clone().unwrap_or_default()))
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_business_zakat() {
        let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
        
        let business = BusinessZakat::builder()
            .cash(dec!(5000.0))
            .inventory(dec!(5000.0))
            .liabilities(dec!(1000.0))
            .hawl(true)
            .build()
            .expect("Valid business");

        let result = business.calculate_zakat(&config).unwrap();

        assert!(result.is_payable);
        assert_eq!(result.net_assets, dec!(9000.0));
        assert_eq!(result.zakat_due, dec!(225.0));
    }

    #[test]
    fn test_business_below_nisab() {
         let config = ZakatConfig { gold_price_per_gram: dec!(100.0), ..Default::default() };
         let business = BusinessZakat::builder()
             .cash(dec!(1000.0))
             .inventory(dec!(1000.0))
             .build()
             .expect("Valid");
         
         let result = business.calculate_zakat(&config).unwrap();
         
         assert!(!result.is_payable);
    }

    #[test]
    fn test_business_specific_case() {
        let config = ZakatConfig { gold_price_per_gram: dec!(1000000.0), ..Default::default() };
        
        let business = BusinessZakat::builder()
            .cash(dec!(100000000.0))
            .liabilities(dec!(20000000.0))
            .hawl(true)
            .build()
            .expect("Valid");
        
        // Use with_config implicitly by passing config
        let result = business.calculate_zakat(&config).unwrap();
        
        assert!(!result.is_payable);
        assert_eq!(result.net_assets, dec!(80000000.0));
    }
}
