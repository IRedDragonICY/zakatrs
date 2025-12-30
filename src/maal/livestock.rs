//! # Fiqh Compliance: Livestock
//!
//! ## Logic
//! - Implements the specific camel age tiers (Bint Makhad, Bint Labun, Hiqqah, Jaza'ah) as defined in the **Letter of Abu Bakr (ra)** (Sahih Bukhari 1454).
//!
//! ## Conditions
//! - **Saimah**: Zakat is only calculated if `grazing_method` is Natural/Saimah, adhering to the majority view (Jumhur) that fodder-fed animals are exempt from Livestock Zakat.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::inputs::IntoZakatDecimal;

pub enum LivestockType {
    Camel,
    Cow,
    Sheep, // Includes Goats
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrazingMethod {
    Saimah,   // Naturally grazed for majority of the year
    Maalufah, // Fed/Fodder provided
}

#[derive(Debug, Clone, Copy)]
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

pub struct LivestockAssets {
    pub count: u32,
    pub animal_type: Option<LivestockType>,
    pub prices: LivestockPrices,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub grazing_method: GrazingMethod,
    pub label: Option<String>,
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
            label: None,
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

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

use crate::config::ZakatConfig;

impl CalculateZakat for LivestockAssets {
    fn calculate_zakat(&self, _config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        let animal_type = self.animal_type.as_ref().ok_or_else(|| 
            ZakatError::InvalidInput("Animal type must be specified".to_string(), self.label.clone())
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
            return Err(ZakatError::ConfigurationError(
                format!("Price for {} must be greater than zero", animal_str), 
                self.label.clone()
            ));
        }

        // Calculate Nisab Count Value for reporting consistency even if not payable
        let nisab_count_val = match animal_type {
            LivestockType::Sheep => Decimal::from(40).checked_mul(single_price).ok_or_else(|| ZakatError::Overflow { operation: "calculate_nisab_value_sheep".to_string(), source: self.label.clone() })?,
            LivestockType::Cow => Decimal::from(30).checked_mul(single_price).ok_or_else(|| ZakatError::Overflow { operation: "calculate_nisab_value_cow".to_string(), source: self.label.clone() })?,
            LivestockType::Camel => Decimal::from(5).checked_mul(single_price).ok_or_else(|| ZakatError::Overflow { operation: "calculate_nisab_value_camel".to_string(), source: self.label.clone() })?,
        };

        if self.grazing_method != GrazingMethod::Saimah {
             return Ok(ZakatDetails::below_threshold(nisab_count_val, crate::types::WealthType::Livestock, "Not Sa'imah (naturally grazed)")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        if !self.hawl_satisfied {
             return Ok(ZakatDetails::below_threshold(nisab_count_val, crate::types::WealthType::Livestock, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        let (zakat_value, nisab_count, heads_due) = match animal_type {
            LivestockType::Sheep => calculate_sheep_zakat(self.count, self.prices.sheep_price)?,
            LivestockType::Cow => calculate_cow_zakat(self.count, self.prices.cow_price)?,
            LivestockType::Camel => calculate_camel_zakat(self.count, &self.prices)?,
        };

        // We construct ZakatDetails.
        // Total Assets = Count * Price (Approx value of herd)
        
        let total_value = Decimal::from(self.count).checked_mul(single_price).ok_or(ZakatError::Overflow { operation: "total_asset_value".to_string(), source: self.label.clone() })?;
        let is_payable = zakat_value > Decimal::ZERO;
        let nisab_threshold = Decimal::from(nisab_count).checked_mul(single_price).ok_or(ZakatError::Overflow { operation: "nisab_threshold".to_string(), source: self.label.clone() })?;

        // Generate description string from heads_due
        let description_parts: Vec<String> = heads_due.iter()
            .map(|(name, count)| format!("{} {}", count, name))
            .collect();
        let description = description_parts.join(", ");

        // Build calculation trace
        let animal_type_str = match animal_type {
            LivestockType::Sheep => "Sheep/Goat",
            LivestockType::Cow => "Cattle",
            LivestockType::Camel => "Camel",
        };
        
        let mut trace = Vec::new();
        trace.push(crate::types::CalculationStep::initial(format!("{} Count", animal_type_str), Decimal::from(self.count)));
        trace.push(crate::types::CalculationStep::info(format!("Animal Type: {}", animal_type_str)));
        trace.push(crate::types::CalculationStep::compare(format!("Nisab Count ({} head)", nisab_count), nisab_threshold));
        if is_payable {
            trace.push(crate::types::CalculationStep::result("Herd Value", total_value));
            trace.push(crate::types::CalculationStep::result(format!("Zakat Due: {}", description), zakat_value));
        } else {
            trace.push(crate::types::CalculationStep::info("Count below Nisab - No Zakat Due"));
        }

        Ok(ZakatDetails {
            total_assets: total_value,
            liabilities_due_now: self.liabilities_due_now,
            net_assets: total_value, 
            nisab_threshold, 
            is_payable,
            zakat_due: zakat_value,
            wealth_type: crate::types::WealthType::Livestock,
            status_reason: None,
            label: self.label.clone(),
            payload: crate::types::PaymentPayload::Livestock { 
                description: description.clone(), 
                heads_due 
            },
            calculation_trace: trace,
        })
    }

    fn get_label(&self) -> Option<String> {
        self.label.clone()
    }
}

#[allow(clippy::type_complexity)]
fn calculate_sheep_zakat(count: u32, price: Decimal) -> Result<(Decimal, u32, Vec<(String, u32)>), ZakatError> {
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

    let zakat_value = Decimal::from(sheep_due)
        .checked_mul(price)
        .ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Sheep Zakat".to_string())))?;
    Ok((zakat_value, nisab, vec![("Sheep".to_string(), sheep_due)]))
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_cow_zakat(count: u32, price: Decimal) -> Result<(Decimal, u32, Vec<(String, u32)>), ZakatError> {
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
        // For counts > 60, the rule is to cover the entire herd count using a combination of 
        // 30s (Tabi') and 40s (Musinnah).
        // This is a partition problem: count = 30*t + 40*m.
        
        // Start with max possible Musinnahs (40s)
        let mut best_m = count / 40;
        let mut best_t = 0;
        let mut found = false;

        // We check if the remainder is divisible by 30.
        // If not, we "swap" one Musinnah (40) into the remainder pool (adding 40 to rem)
        // and check if the new remainder is divisible by 30.
        // Since 3 * 40 = 120 = 4 * 30, we only need to check at most 3 swaps before the pattern repeats/cycles.
        
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
            
            if best_m == 0 { break; } // Cannot swap further
            best_m -= 1;
        }

        if found {
            musinnah = best_m;
            tabi = best_t;
        } else {
            // Fallback: If no perfect partition exists (rare/impossible for large numbers),
            // we default to prioritizing 40s and covering remainder logic or just best effort.
            // For standard Zakat rules, large herds are usually partitioned.
            // Default best effort: Max 40s.
            musinnah = count / 40;
            let rem = count % 40;
            if rem >= 30 { tabi = 1; }
        }
    }

    // Value estimation based on pricing ratios relative to a standard cow price:
    // Tabi (1yo) is estimated at 0.7x of standard price.
    // Musinnah (2yo) is estimated at 1.0x of standard price.
    let val_tabi = price.checked_mul(dec!(0.7)).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Cow Zakat".to_string())))?;
    let val_musinnah = price;
    
    let tabi_total = Decimal::from(tabi).checked_mul(val_tabi).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Cow Zakat".to_string())))?;
    let musinnah_total = Decimal::from(musinnah).checked_mul(val_musinnah).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Cow Zakat".to_string())))?;
    let total_zakat_val = tabi_total.checked_add(musinnah_total).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Cow Zakat".to_string())))?;
    
    let mut parts = Vec::new();
    if tabi > 0 { parts.push(("Tabi'".to_string(), tabi)); }
    if musinnah > 0 { parts.push(("Musinnah".to_string(), musinnah)); }

    Ok((total_zakat_val, nisab, parts))
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_camel_zakat(count: u32, prices: &LivestockPrices) -> Result<(Decimal, u32, Vec<(String, u32)>), ZakatError> {
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
        // Recursive logic for 121+:
        // 1 Bint Labun (40s), 1 Hiqqah (50s).
        // Maximize Hiqqah (50s) as they are larger/more valuable.
        
        let mut best_h = count / 50;
        let mut best_b = 0;
        let mut found = false;

        // Swap Strategy O(1): Try converting a 50 into 40s.
        // 4 * 50 = 200 = 5 * 40. Relies on LCM(40, 50) = 200.
        // Max swaps needed = 4.
        
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
        
        // Fallback or found
        if !found {
            // Default approach for non-perfect fit
             best_h = count / 50;
             let rem = count % 50;
             if rem >= 40 { best_b = 1; }
        }

        (0, 0, best_b, best_h, 0)
    };

    // Valuation Ratios:
    // Sheep = sheep_price
    // Bint Makhad (1yo) = 0.5x camel_price 
    // Bint Labun (2yo) = 0.75x camel_price
    // Hiqqah (3yo) = 1.0x camel_price (Prime)
    // Jazaah (4yo) = 1.25x camel_price
    
    // Pricing implementation:
    let v_sheep = prices.sheep_price;
    let v_camel = prices.camel_price; 
    let v_bm = v_camel.checked_mul(dec!(0.5)).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?;
    let v_bl = v_camel.checked_mul(dec!(0.75)).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?;
    let v_hq = v_camel;
    let v_jz = v_camel.checked_mul(dec!(1.25)).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?;
    
    let total = Decimal::from(sheep).checked_mul(v_sheep).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?
        .checked_add(Decimal::from(b_makhad).checked_mul(v_bm).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?
        .checked_add(Decimal::from(b_labun).checked_mul(v_bl).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?
        .checked_add(Decimal::from(hiqqah).checked_mul(v_hq).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?
        .checked_add(Decimal::from(jazaah).checked_mul(v_jz).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?).ok_or_else(|| ZakatError::CalculationError("Mathematical error in livestock logic (Internal)".to_string(), Some("Camel Zakat".to_string())))?;
        
    let mut parts = Vec::new();
    if sheep > 0 { parts.push(("Sheep".to_string(), sheep)); }
    if b_makhad > 0 { parts.push(("Bint Makhad".to_string(), b_makhad)); }
    if b_labun > 0 { parts.push(("Bint Labun".to_string(), b_labun)); }
    if hiqqah > 0 { parts.push(("Hiqqah".to_string(), hiqqah)); }
    if jazaah > 0 { parts.push(("Jaza'ah".to_string(), jazaah)); }

    Ok((total, nisab, parts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheep() {
        let prices = LivestockPrices::new().sheep_price(dec!(100.0));
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
        assert_eq!(res.zakat_due, dec!(100.0));

        let stock = LivestockAssets::new()
            .count(120)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, dec!(100.0));
        
         // 121-200 -> 2 sheep
        let stock = LivestockAssets::new()
            .count(121)
            .animal_type(LivestockType::Sheep)
            .prices(prices)
            .hawl(true);
        let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, dec!(200.0));
    }

    #[test]
    fn test_camels() {
         let prices = LivestockPrices::new()
            .camel_price(dec!(1000.0))
            .sheep_price(dec!(100.0));
         
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
         assert_eq!(res.zakat_due, dec!(100.0)); // 1 sheep value
         
         // 25-35 -> 1 Bint Makhad (Camel)
         let stock = LivestockAssets::new()
            .count(25)
            .animal_type(LivestockType::Camel)
            .prices(prices)
            .hawl(true);
         let res = stock.calculate_zakat(&ZakatConfig::default()).unwrap();
         assert_eq!(res.zakat_due, dec!(500.0)); // 1 Bint Makhad (0.5x camel_price)
    }

    #[test]
    fn test_cows() {
         let prices = LivestockPrices::new().cow_price(dec!(500.0));
         
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
         assert_eq!(res.zakat_due, dec!(350.0)); // 1 Tabi (0.7x cow_price)
    }

    #[test]
    fn test_maalufah_below_threshold() {
        let prices = LivestockPrices::new().sheep_price(dec!(100.0));
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
            .cow_price(dec!(500.0));

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
        assert!(details.zakat_due > dec!(0));
        
        // Value sanity check: 100M cows * $500 = $50B. Zakat should be roughly 2.5% value?
        // Actually Livestock Zakat is approx 2.5% value but calculated via heads.
        // 100M cows -> ~2.5M heads due. 
        // 2.5M * $500 = $1.25B approx.
        // Let's just ensure it calculated "something" reasonable.
        assert!(details.zakat_due > dec!(1_000_000_000));
    }
}
