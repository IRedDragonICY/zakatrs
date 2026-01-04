//! # Fiqh Compliance: Livestock
//!
//! ## Logic
//! - Implements the specific camel age tiers (Bint Makhad, Bint Labun, Hiqqah, Jaza'ah) as defined in the **Letter of Abu Bakr (ra)** (Sahih Bukhari 1454).
//!
//! ## Conditions
//! - **Saimah**: Zakat is only calculated if `grazing_method` is Natural/Saimah, adhering to the majority view (Jumhur) that fodder-fed animals are exempt from Livestock Zakat.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError, InvalidInputDetails, ErrorDetails, LivestockAge, LivestockKind, LivestockDueItem};
use serde::{Serialize, Deserialize};
use crate::traits::{CalculateZakat, ZakatConfigArgument};
use crate::inputs::IntoZakatDecimal;
use crate::math::ZakatDecimal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum LivestockType {
    Camel,
    Cow,
    Sheep, // Includes Goats
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GrazingMethod {
    #[default]
    Saimah,   // Naturally grazed for majority of the year
    Maalufah, // Fed/Fodder provided
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LivestockPrices {
    pub sheep_price: Decimal,
    pub cow_price: Decimal, // For Tabi/Musinnah avg or simplified
    pub camel_price: Decimal,
}

impl Default for LivestockPrices {
    fn default() -> Self {
        Self {
            sheep_price: Decimal::ZERO,
            cow_price: Decimal::ZERO,
            camel_price: Decimal::ZERO,
        }
    }
}

impl LivestockPrices {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sheep_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.sheep_price = p;
        }
        self
    }

    pub fn cow_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.cow_price = p;
        }
        self
    }

    pub fn camel_price(mut self, price: impl IntoZakatDecimal) -> Self {
         if let Ok(p) = price.into_zakat_decimal() {
            self.camel_price = p;
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LivestockAssets {
    pub count: u32,
    pub animal_type: Option<LivestockType>,
    pub prices: LivestockPrices,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub grazing_method: GrazingMethod,
    pub is_working_animal: bool, // Exemption for Awamil
    pub label: Option<String>,
    pub id: uuid::Uuid,
}

impl Default for LivestockAssets {
    fn default() -> Self {
        Self {
            count: 0,
            animal_type: None,
            prices: LivestockPrices::default(),
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            grazing_method: GrazingMethod::Saimah,
            is_working_animal: false,
            label: None,
            id: uuid::Uuid::new_v4(),
        }
    }
}

impl LivestockAssets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }

    pub fn animal_type(mut self, animal_type: LivestockType) -> Self {
        self.animal_type = Some(animal_type);
        self
    }

    pub fn prices(mut self, prices: LivestockPrices) -> Self {
        self.prices = prices;
        self
    }

    pub fn debt(mut self, debt: impl IntoZakatDecimal) -> Self {
        if let Ok(d) = debt.into_zakat_decimal() {
            self.liabilities_due_now = d;
        }
        self
    }

    pub fn hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn grazing(mut self, method: GrazingMethod) -> Self {
        self.grazing_method = method;
        self
    }

    /// Sets whether the animals are working animals (Al-Awamil).
    /// Working animals (plowing, transport, irrigation) are exempt from Zakat.
    pub fn working_animal(mut self, is_working: bool) -> Self {
        self.is_working_animal = is_working;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_sheep_price(mut self, price: impl IntoZakatDecimal) -> Self {
        self.prices = self.prices.sheep_price(price);
        self
    }

    pub fn with_cow_price(mut self, price: impl IntoZakatDecimal) -> Self {
        self.prices = self.prices.cow_price(price);
        self
    }

    pub fn with_camel_price(mut self, price: impl IntoZakatDecimal) -> Self {
        self.prices = self.prices.camel_price(price);
        self
    }
    
    pub fn validate(&self) -> Result<(), ZakatError> {
        Ok(())
    }
}



impl CalculateZakat for LivestockAssets {
    fn validate_input(&self) -> Result<(), ZakatError> {
        self.validate()
    }

    fn calculate_zakat<C: ZakatConfigArgument>(&self, config: C) -> Result<ZakatDetails, ZakatError> {
        // Early return optimization for zero count
        if self.count == 0 {
            return Ok(ZakatDetails::below_threshold(Decimal::ZERO, crate::types::WealthType::Livestock, "Count is zero")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        
        let config_cow = config.resolve_config();
        let _config_ref = config_cow.as_ref();

        let animal_type = self.animal_type.as_ref().ok_or_else(|| 
            ZakatError::InvalidInput(Box::new(InvalidInputDetails {
                field: "animal_type".to_string(),
                value: "None".to_string(),
                reason_key: "error-type-required".to_string(),
                source_label: self.label.clone(),
                suggestion: Some("Specify the animal type: Camel, Cow, or Sheep.".to_string()),
                ..Default::default()
            }))
        )?;

        // Validate price for the specific animal type
        let single_price = match animal_type {
            LivestockType::Sheep => self.prices.sheep_price,
            LivestockType::Cow => self.prices.cow_price,
            LivestockType::Camel => self.prices.camel_price,
        };

        if single_price <= Decimal::ZERO {
            let animal_str = match animal_type {
                LivestockType::Sheep => "Sheep",
                LivestockType::Cow => "Cow",
                LivestockType::Camel => "Camel",
            };
            return Err(ZakatError::ConfigurationError(Box::new(ErrorDetails {
                code: crate::types::ZakatErrorCode::ConfigError,
                reason_key: "error-price-zero".to_string(),
                args: Some(std::collections::HashMap::from([("animal".to_string(), animal_str.to_string())])), 
                source_label: self.label.clone(),
                suggestion: Some(format!("Set a price for {} using prices().", animal_str)),
                ..Default::default()
            })));
        }

        // Calculate Nisab Count Value for reporting consistency even if not payable
        let nisab_count_val = match animal_type {
            LivestockType::Sheep => ZakatDecimal::new(Decimal::from(40)).safe_mul(single_price)?.with_source(self.label.clone()),
            LivestockType::Cow => ZakatDecimal::new(Decimal::from(30)).safe_mul(single_price)?.with_source(self.label.clone()),
            LivestockType::Camel => ZakatDecimal::new(Decimal::from(5)).safe_mul(single_price)?.with_source(self.label.clone()),
        };

        // Fiqh Rule: Working animals (Al-Awamil) are Exempt
        if self.is_working_animal {
             return Ok(ZakatDetails::below_threshold(
                *nisab_count_val, 
                crate::types::WealthType::Livestock, 
                "Working animals (Awamil) are exempt"
            ).with_label(self.label.clone().unwrap_or_default()));
        }

        if self.grazing_method != GrazingMethod::Saimah {
             return Ok(ZakatDetails::below_threshold(*nisab_count_val, crate::types::WealthType::Livestock, "Not Sa'imah (naturally grazed)")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        if !self.hawl_satisfied {
             return Ok(ZakatDetails::below_threshold(*nisab_count_val, crate::types::WealthType::Livestock, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        // Note: translator is available via config_ref if needed for trace messages

        let (zakat_value, nisab_count, heads_due) = match animal_type {
            LivestockType::Sheep => calculate_sheep_zakat(self.count, self.prices.sheep_price)?,
            LivestockType::Cow => calculate_cow_zakat(self.count, self.prices.cow_price)?,
            LivestockType::Camel => calculate_camel_zakat(self.count, &self.prices)?,
        };

        // We construct ZakatDetails.
        // Total Assets = Count * Price (Approx value of herd)
        
        let total_value = ZakatDecimal::new(Decimal::from(self.count))
            .safe_mul(single_price)?
            .with_source(self.label.clone());
            
        let is_payable = zakat_value > Decimal::ZERO;
        
        let nisab_threshold = ZakatDecimal::new(Decimal::from(nisab_count))
            .safe_mul(single_price)?
            .with_source(self.label.clone());

        // Generate description string from heads_due using PaymentPayload helper
        let payload = crate::types::PaymentPayload::Livestock { heads_due: heads_due.clone() };
        let description = payload.livestock_description().unwrap_or_default();

        // Build calculation trace
        // Use translator for the animal type itself in trace? 
        // Plan asked for output payload localization (heads_due). Trace usually stays English or low priority, 
        // but let's localize animal type if easy. 
        // For now, focusing on the PaymentPayload as requested.
        let animal_type_str = match animal_type {
            LivestockType::Sheep => "Sheep/Goat",
            LivestockType::Cow => "Cattle",
            LivestockType::Camel => "Camel",
        };
        
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial("step-livestock-count", format!("{} Count", animal_type_str), Decimal::from(self.count))
            .with_args(std::collections::HashMap::from([("type".to_string(), animal_type_str.to_string())])));
        
        // ... (truncated trace logic same as before) ...
        trace.push(crate::types::CalculationStep::info("info-animal-type", format!("Animal Type: {}", animal_type_str))
             .with_args(std::collections::HashMap::from([("type".to_string(), animal_type_str.to_string())])));
        
        trace.push(crate::types::CalculationStep::compare("step-nisab-check-count", format!("Nisab Count ({} head)", nisab_count), *nisab_threshold)
             .with_args(std::collections::HashMap::from([("count".to_string(), nisab_count.to_string())])));

        if is_payable {
            trace.push(crate::types::CalculationStep::result("step-herd-value", "Herd Value", *total_value));
            trace.push(crate::types::CalculationStep::result("step-zakat-due-desc", format!("Zakat Due: {}", description), zakat_value)
                 .with_args(std::collections::HashMap::from([("description".to_string(), description.clone())])));
        } else {
            trace.push(crate::types::CalculationStep::info("status-exempt", "Count below Nisab - No Zakat Due"));
        }

        #[allow(deprecated)]
        Ok(ZakatDetails {
            total_assets: *total_value,
            liabilities_due_now: self.liabilities_due_now,
            liabilities: Vec::new(),
            net_assets: *total_value, 
            nisab_threshold: *nisab_threshold, 
            is_payable,
            zakat_due: zakat_value,
            wealth_type: crate::types::WealthType::Livestock,
            status_reason: None,
            label: self.label.clone(),
            payload: crate::types::PaymentPayload::Livestock { heads_due },
            calculation_trace: crate::types::CalculationTrace(trace),
            warnings: Vec::new(),
            structured_warnings: Vec::new(),
            recommendation: if is_payable { 
                crate::types::ZakatRecommendation::Obligatory 
            } else { 
                crate::types::ZakatRecommendation::None 
            },
        })
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }

    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}

#[allow(clippy::type_complexity)]
fn calculate_sheep_zakat(count: u32, price: Decimal) -> Result<(Decimal, u32, Vec<LivestockDueItem>), ZakatError> {
    let nisab = 40;
    if count < 40 {
        return Ok((Decimal::ZERO, nisab, vec![]));
    }
    
    let sheep_due = if count <= 120 {
        1
    } else if count <= 200 {
        2
    } else if count <= 300 {
        3
    } else {
        // Above 300: 1 sheep for every 100 sheep.
        count / 100
    };

    let zakat_value = ZakatDecimal::new(Decimal::from(sheep_due))
        .safe_mul(price)?
        .with_source(Some("Sheep Zakat".to_string()));
    
    Ok((*zakat_value, nisab, vec![LivestockDueItem::new(sheep_due, LivestockAge::Jadha, LivestockKind::Sheep)]))
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_cow_zakat(count: u32, price: Decimal) -> Result<(Decimal, u32, Vec<LivestockDueItem>), ZakatError> {
    let nisab = 30;
    if count < 30 {
        return Ok((Decimal::ZERO, nisab, vec![]));
    }

    // Cows Zakat Logic:
    // 30-39: 1 Tabi (Yearling)
    // 40-59: 1 Musinnah (2yo)
    // 60+: Combination of 30s (Tabi) and 40s (Musinnah) to cover the total count.
    
    let mut tabi = 0;
    let mut musinnah = 0;

    if count < 60 {
        if count <= 39 { tabi = 1; }
        else { musinnah = 1; }
    } else {
        // O(1) Optimization: Swap Strategy
        let mut best_m = count / 40;
        let mut best_t = 0;
        let mut found = false;

        for _ in 0..=3 {
            let used_count = best_m * 40;
            if used_count <= count {
                let rem = count - used_count;
                if rem % 30 == 0 {
                    best_t = rem / 30;
                    found = true;
                    break;
                }
            }
            
            if best_m == 0 { break; }
            best_m -= 1;
        }

        if found {
            musinnah = best_m;
            tabi = best_t;
        } else {
            musinnah = count / 40;
            let rem = count % 40;
            if rem >= 30 { tabi = 1; }
        }
    }

    // Value estimation
    let val_tabi = ZakatDecimal::new(price).safe_mul(dec!(0.7))?.with_source(Some("Cow Zakat".to_string()));
    let val_musinnah = price;
    
    let tabi_total = ZakatDecimal::new(Decimal::from(tabi)).safe_mul(*val_tabi)?.with_source(Some("Cow Zakat".to_string()));
    let musinnah_total = ZakatDecimal::new(Decimal::from(musinnah)).safe_mul(val_musinnah)?.with_source(Some("Cow Zakat".to_string()));
    let total_zakat_val = tabi_total.safe_add(*musinnah_total)?.with_source(Some("Cow Zakat".to_string()));
    
    let mut parts = Vec::new();
    if tabi > 0 { 
        parts.push(LivestockDueItem::new(tabi, LivestockAge::Tabi, LivestockKind::Cow)); 
    }
    if musinnah > 0 { 
        parts.push(LivestockDueItem::new(musinnah, LivestockAge::Musinnah, LivestockKind::Cow)); 
    }

    Ok((*total_zakat_val, nisab, parts))
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_camel_zakat(count: u32, prices: &LivestockPrices) -> Result<(Decimal, u32, Vec<LivestockDueItem>), ZakatError> {
    let nisab = 5;
    if count < 5 {
        return Ok((Decimal::ZERO, nisab, vec![]));
    }
    
    // 5-24: Sheep logic (standard)
    // 25-120: Discrete Camel ranges
    // 121+: 1 Bint Labun per 40, 1 Hiqqah per 50.
    
    let (sheep, b_makhad, b_labun, hiqqah, jazaah) = if count < 25 {
        let s = if count < 10 { 1 } else if count < 15 { 2 } else if count < 20 { 3 } else { 4 };
        (s, 0, 0, 0, 0)
    } else if count <= 35 { (0, 1, 0, 0, 0) }
    else if count <= 45 { (0, 0, 1, 0, 0) }
    else if count <= 60 { (0, 0, 0, 1, 0) }
    else if count <= 75 { (0, 0, 0, 0, 1) }
    else if count <= 90 { (0, 0, 2, 0, 0) }
    else if count <= 120 { (0, 0, 0, 2, 0) }
    else {
        // Recursive logic for 121+
        let mut best_h = count / 50;
        let mut best_b = 0;
        let mut found = false;

        for _ in 0..=4 {
            let used_count = best_h * 50;
            if used_count <= count {
                let rem = count - used_count;
                if rem % 40 == 0 {
                    best_b = rem / 40;
                    found = true;
                    break;
                }
            }
            if best_h == 0 { break; }
            best_h -= 1;
        }
        
        if !found {
             best_h = count / 50;
             let rem = count % 50;
             if rem >= 40 { best_b = 1; }
        }

        (0, 0, best_b, best_h, 0)
    };

    // Pricing
    let v_sheep = prices.sheep_price;
    let v_camel = prices.camel_price; 
    let v_bm = ZakatDecimal::new(v_camel).safe_mul(dec!(0.5))?.with_source(Some("Camel Zakat".to_string()));
    let v_bl = ZakatDecimal::new(v_camel).safe_mul(dec!(0.75))?.with_source(Some("Camel Zakat".to_string()));
    let v_hq = v_camel;
    let v_jz = ZakatDecimal::new(v_camel).safe_mul(dec!(1.25))?.with_source(Some("Camel Zakat".to_string()));
    
    let total = ZakatDecimal::new(Decimal::from(sheep)).safe_mul(v_sheep)?
        .safe_add(ZakatDecimal::new(Decimal::from(b_makhad)).safe_mul(*v_bm)?.0)?
        .safe_add(ZakatDecimal::new(Decimal::from(b_labun)).safe_mul(*v_bl)?.0)?
        .safe_add(ZakatDecimal::new(Decimal::from(hiqqah)).safe_mul(v_hq)?.0)?
        .safe_add(ZakatDecimal::new(Decimal::from(jazaah)).safe_mul(*v_jz)?.0)?
        .with_source(Some("Camel Zakat".to_string()));
        
    let mut parts = Vec::new();
    if sheep > 0 { 
        parts.push(LivestockDueItem::new(sheep, LivestockAge::Jadha, LivestockKind::Sheep)); 
    }
    if b_makhad > 0 { 
        parts.push(LivestockDueItem::new(b_makhad, LivestockAge::BintMakhad, LivestockKind::Camel)); 
    }
    if b_labun > 0 { 
        parts.push(LivestockDueItem::new(b_labun, LivestockAge::BintLabun, LivestockKind::Camel)); 
    }
    if hiqqah > 0 { 
        parts.push(LivestockDueItem::new(hiqqah, LivestockAge::Hiqqah, LivestockKind::Camel)); 
    }
    if jazaah > 0 { 
        parts.push(LivestockDueItem::new(jazaah, LivestockAge::Jazaah, LivestockKind::Camel)); 
    }

    Ok((*total, nisab, parts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZakatConfig;

    #[test]
    fn test_sheep() {
        let prices = LivestockPrices::new().sheep_price(100.0);
        // 1-39 -> 0
        let stock = LivestockAssets::new()
            .count(39)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(!res.is_payable);

        // 40-120 -> 1 sheep
        let stock = LivestockAssets::new()
            .count(40)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, Decimal::from(100));

        let stock = LivestockAssets::new()
            .count(120)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, Decimal::from(100));
        
         // 121-200 -> 2 sheep
        let stock = LivestockAssets::new()
            .count(121)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, Decimal::from(200));
    }

    #[test]
    fn test_camels() {
         let prices = LivestockPrices::new()
            .camel_price(1000.0)
            .sheep_price(100.0);
         
         // 1-4 -> 0
         let stock = LivestockAssets::new()
            .count(4)
            .animal_type(LivestockType::Camel)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(!res.is_payable);

         // 5-9 -> 1 sheep
         let stock = LivestockAssets::new()
            .count(5)
            .animal_type(LivestockType::Camel)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, Decimal::from(100)); // 1 sheep value
         
         // 25-35 -> 1 Bint Makhad (Camel)
         let stock = LivestockAssets::new()
            .count(25)
            .animal_type(LivestockType::Camel)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert_eq!(res.zakat_due, Decimal::from(500)); // 1 Bint Makhad (0.5x camel_price)
    }

    #[test]
    fn test_cows() {
         let prices = LivestockPrices::new().cow_price(500.0);
         
         // 1-29 -> 0
         let stock = LivestockAssets::new()
            .count(29)
            .animal_type(LivestockType::Cow)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(!res.is_payable);

         // 30-39 -> 1 Tabi' (implied 1 year old cow, assumed base price here)
         // For simplicity using cow_price. In reality Tabi' vs Musinnah prices differ.
         let stock = LivestockAssets::new()
            .count(30)
            .animal_type(LivestockType::Cow)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, Decimal::from(350)); // 1 Tabi (0.7x cow_price)
    }

    #[test]
    fn test_maalufah_below_threshold() {
        let prices = LivestockPrices::new().sheep_price(100.0);
        // 50 Sheep (usually payable) but Feed-lot (Maalufah)
        let stock = LivestockAssets::new()
            .count(50)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .grazing(GrazingMethod::Maalufah)
            .hawl(true);
            
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(!res.is_payable);
        assert_eq!(res.status_reason, Some("Not Sa'imah (naturally grazed)".to_string()));
    }

    #[test]
    fn test_large_number_success() {
        let prices = LivestockPrices::new()
            .cow_price(500.0);

        // 100M + 1 cows. Previously failed due to complexity/iteration limit.
        // Now should pass instantly with O(1) logic.
        
        let stock_large = LivestockAssets::new()
            .count(100_000_001)
            .animal_type(LivestockType::Cow)
            .prices(prices);
            
        let res_large = stock_large.calculate_zakat(&ZakatConfig::default());
        
        // Should NOT be an error now
        assert!(res_large.is_ok()); 
        let details = res_large.unwrap();
        assert!(details.is_payable);
        assert!(details.zakat_due > Decimal::ZERO);
        
        // Value sanity check: 100M cows * $500 = $50B. Zakat should be roughly 2.5% value.
        // Actually Livestock Zakat is approx 2.5% value but calculated via heads.
        // 100M cows -> ~2.5M heads due. 
        // 2.5M * $500 = $1.25B approx.
        // Verify that the result is within the expected order of magnitude.
        assert!(details.zakat_due > dec!(1_000_000_000));
    }
}
