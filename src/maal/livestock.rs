use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;
use crate::inputs::IntoZakatDecimal;
use crate::builder::AssetBuilder;

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

pub struct LivestockAssets {
    pub count: u32,
    pub animal_type: LivestockType,
    pub prices: LivestockPrices,
    pub liabilities_due_now: Decimal,
    pub hawl_satisfied: bool,
    pub grazing_method: GrazingMethod,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct LivestockPrices {
    pub sheep_price: Decimal,
    pub cow_price: Decimal, // For Tabi/Musinnah avg or simplified
    pub camel_price: Decimal,
}

impl LivestockPrices {
    pub fn builder() -> LivestockPricesBuilder {
        LivestockPricesBuilder::default()
    }

    /// Deprecated: Use `LivestockPrices::builder()` instead.
    #[deprecated(since = "0.2.1", note = "Use `LivestockPrices::builder()` instead")]
    pub fn new(
        sheep_price: impl IntoZakatDecimal,
        cow_price: impl IntoZakatDecimal,
        camel_price: impl IntoZakatDecimal,
    ) -> Result<Self, ZakatError> {
        Self::builder()
            .sheep_price(sheep_price)
            .cow_price(cow_price)
            .camel_price(camel_price)
            .build()
    }
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

#[derive(Default)]
pub struct LivestockPricesBuilder {
    sheep_price: Option<Decimal>,
    cow_price: Option<Decimal>,
    camel_price: Option<Decimal>,
}

impl LivestockPricesBuilder {
    pub fn sheep_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.sheep_price = Some(p);
        }
        self
    }

    pub fn cow_price(mut self, price: impl IntoZakatDecimal) -> Self {
        if let Ok(p) = price.into_zakat_decimal() {
            self.cow_price = Some(p);
        }
        self
    }

    pub fn camel_price(mut self, price: impl IntoZakatDecimal) -> Self {
         if let Ok(p) = price.into_zakat_decimal() {
            self.camel_price = Some(p);
        }
        self
    }
}

impl AssetBuilder<LivestockPrices> for LivestockPricesBuilder {
    fn build(self) -> Result<LivestockPrices, ZakatError> {
        // We require at least one price to be set or explicit 0.
        // But for safety, let's just ensure if they ARE set, they are non-negative.
        // If not set, they default to 0, which is technically safer than "Random Default" but user should set them.
        
        let s = self.sheep_price.unwrap_or(Decimal::ZERO);
        let c = self.cow_price.unwrap_or(Decimal::ZERO);
        let m = self.camel_price.unwrap_or(Decimal::ZERO); // m for camel (ibl)

        if s < Decimal::ZERO || c < Decimal::ZERO || m < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Livestock prices must be non-negative".to_string(), None));
        }

        Ok(LivestockPrices {
            sheep_price: s,
            cow_price: c,
            camel_price: m,
        })
    }
}



impl LivestockAssets {
    pub fn new(
        count: u32,
        animal_type: LivestockType,
        prices: LivestockPrices,
    ) -> Self {
        Self {
            count,
            animal_type,
            prices,
            liabilities_due_now: Decimal::ZERO,
            hawl_satisfied: true,
            grazing_method: GrazingMethod::Saimah, // Default to Zakatable state (Saimah)
            label: None,
        }
    }

    pub fn with_debt_due_now(mut self, debt: impl IntoZakatDecimal) -> Result<Self, ZakatError> {
        self.liabilities_due_now = debt.into_zakat_decimal()?;
        Ok(self)
    }

    pub fn with_hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn with_grazing_method(mut self, method: GrazingMethod) -> Self {
        self.grazing_method = method;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

use crate::config::ZakatConfig;

impl CalculateZakat for LivestockAssets {
    fn calculate_zakat(&self, _config: &ZakatConfig) -> Result<ZakatDetails, ZakatError> {
        // Validate price for the specific animal type
        let single_price = match self.animal_type {
            LivestockType::Sheep => self.prices.sheep_price,
            LivestockType::Cow => self.prices.cow_price,
            LivestockType::Camel => self.prices.camel_price,
        };

        if single_price <= Decimal::ZERO {
            let animal_str = match self.animal_type {
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
        
        let nisab_count_val = match self.animal_type {
            LivestockType::Sheep => Decimal::from(40).checked_mul(single_price).unwrap_or(Decimal::MAX),
            LivestockType::Cow => Decimal::from(30).checked_mul(single_price).unwrap_or(Decimal::MAX),
            LivestockType::Camel => Decimal::from(5).checked_mul(single_price).unwrap_or(Decimal::MAX),
        };

        if self.grazing_method != GrazingMethod::Saimah {
             return Ok(ZakatDetails::below_threshold(nisab_count_val, crate::types::WealthType::Livestock, "Not Sa'imah (naturally grazed)")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        if !self.hawl_satisfied {
             return Ok(ZakatDetails::below_threshold(nisab_count_val, crate::types::WealthType::Livestock, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }

        let (zakat_value, nisab_count, heads_due) = match self.animal_type {
            LivestockType::Sheep => calculate_sheep_zakat(self.count, self.prices.sheep_price),
            LivestockType::Cow => calculate_cow_zakat(self.count, self.prices.cow_price),
            LivestockType::Camel => calculate_camel_zakat(self.count, &self.prices),
        };

        // We construct ZakatDetails.
        // Total Assets = Count * Price (Approx value of herd)
        
        let total_value = Decimal::from(self.count).checked_mul(single_price).ok_or(ZakatError::CalculationError("Total asset value overflow".to_string(), None))?;
        let is_payable = zakat_value > Decimal::ZERO;
        let nisab_threshold = Decimal::from(nisab_count).checked_mul(single_price).unwrap_or(Decimal::MAX);

        // Generate description string from heads_due
        let description_parts: Vec<String> = heads_due.iter()
            .map(|(name, count)| format!("{} {}", count, name))
            .collect();
        let description = description_parts.join(", ");

        // Build calculation trace
        let animal_type_str = match self.animal_type {
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

fn calculate_sheep_zakat(count: u32, price: Decimal) -> (Decimal, u32, Vec<(String, u32)>) {
    let nisab = 40;
    if count < 40 {
        return (Decimal::ZERO, nisab, vec![]);
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
        .unwrap_or(Decimal::MAX);
    (zakat_value, nisab, vec![("Sheep".to_string(), sheep_due)])
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_cow_zakat(count: u32, price: Decimal) -> (Decimal, u32, Vec<(String, u32)>) {
    let nisab = 30;
    if count < 30 {
        return (Decimal::ZERO, nisab, vec![]);
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
    let val_tabi = price.checked_mul(dec!(0.7)).unwrap_or(Decimal::MAX);
    let val_musinnah = price;
    
    let tabi_total = Decimal::from(tabi).checked_mul(val_tabi).unwrap_or(Decimal::MAX);
    let musinnah_total = Decimal::from(musinnah).checked_mul(val_musinnah).unwrap_or(Decimal::MAX);
    let total_zakat_val = tabi_total.checked_add(musinnah_total).unwrap_or(Decimal::MAX);
    
    let mut parts = Vec::new();
    if tabi > 0 { parts.push(("Tabi'".to_string(), tabi)); }
    if musinnah > 0 { parts.push(("Musinnah".to_string(), musinnah)); }

    (total_zakat_val, nisab, parts)
}

#[allow(clippy::type_complexity)]
#[allow(clippy::manual_is_multiple_of)]
fn calculate_camel_zakat(count: u32, prices: &LivestockPrices) -> (Decimal, u32, Vec<(String, u32)>) {
    let nisab = 5;
    if count < 5 {
        return (Decimal::ZERO, nisab, vec![]);
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
    let v_bm = v_camel.checked_mul(dec!(0.5)).unwrap_or(Decimal::MAX);
    let v_bl = v_camel.checked_mul(dec!(0.75)).unwrap_or(Decimal::MAX);
    let v_hq = v_camel;
    let v_jz = v_camel.checked_mul(dec!(1.25)).unwrap_or(Decimal::MAX);
    
    let total = Decimal::from(sheep).checked_mul(v_sheep).unwrap_or(Decimal::MAX)
        .checked_add(Decimal::from(b_makhad).checked_mul(v_bm).unwrap_or(Decimal::MAX)).unwrap_or(Decimal::MAX)
        .checked_add(Decimal::from(b_labun).checked_mul(v_bl).unwrap_or(Decimal::MAX)).unwrap_or(Decimal::MAX)
        .checked_add(Decimal::from(hiqqah).checked_mul(v_hq).unwrap_or(Decimal::MAX)).unwrap_or(Decimal::MAX)
        .checked_add(Decimal::from(jazaah).checked_mul(v_jz).unwrap_or(Decimal::MAX)).unwrap_or(Decimal::MAX);
        
    let mut parts = Vec::new();
    if sheep > 0 { parts.push(("Sheep".to_string(), sheep)); }
    if b_makhad > 0 { parts.push(("Bint Makhad".to_string(), b_makhad)); }
    if b_labun > 0 { parts.push(("Bint Labun".to_string(), b_labun)); }
    if hiqqah > 0 { parts.push(("Hiqqah".to_string(), hiqqah)); }
    if jazaah > 0 { parts.push(("Jaza'ah".to_string(), jazaah)); }

    (total, nisab, parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheep() {
        let prices = LivestockPrices { sheep_price: dec!(100.0), ..Default::default() };
        // 1-39 -> 0
        let stock = LivestockAssets::new(39, LivestockType::Sheep, prices);
        let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(!res.is_payable);

        // 40-120 -> 1 sheep
        let stock = LivestockAssets::new(40, LivestockType::Sheep, prices);
        let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(res.is_payable);
        assert_eq!(res.zakat_due, dec!(100.0));

        let stock = LivestockAssets::new(120, LivestockType::Sheep, prices);
        let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, dec!(100.0));
        
         // 121-200 -> 2 sheep
        let stock = LivestockAssets::new(121, LivestockType::Sheep, prices);
        let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
        assert_eq!(res.zakat_due, dec!(200.0));
    }

    #[test]
    fn test_camels() {
         let prices = LivestockPrices { camel_price: dec!(1000.0), sheep_price: dec!(100.0), ..Default::default() };
         
         // 1-4 -> 0
         let stock = LivestockAssets::new(4, LivestockType::Camel, prices);
         let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(!res.is_payable);

         // 5-9 -> 1 sheep
         let stock = LivestockAssets::new(5, LivestockType::Camel, prices);
         let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, dec!(100.0)); // 1 sheep value
         
         // 25-35 -> 1 Bint Makhad (Camel)
         let stock = LivestockAssets::new(25, LivestockType::Camel, prices);
         let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
         assert_eq!(res.zakat_due, dec!(500.0)); // 1 Bint Makhad (0.5x camel_price)
    }

    #[test]
    fn test_cows() {
         let prices = LivestockPrices { cow_price: dec!(500.0), ..Default::default() };
         
         // 1-29 -> 0
         let stock = LivestockAssets::new(29, LivestockType::Cow, prices);
         let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(!res.is_payable);

         // 30-39 -> 1 Tabi' (implied 1 year old cow, assumed base price here)
         // For simplicity using cow_price. In reality Tabi' vs Musinnah prices differ.
         let stock = LivestockAssets::new(30, LivestockType::Cow, prices);
         let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
         assert!(res.is_payable);
         assert_eq!(res.zakat_due, dec!(350.0)); // 1 Tabi (0.7x cow_price)
    }

    #[test]
    fn test_maalufah_below_threshold() {
        let prices = LivestockPrices { sheep_price: dec!(100.0), ..Default::default() };
        // 50 Sheep (usually payable) but Feed-lot (Maalufah)
        let stock = LivestockAssets::new(50, LivestockType::Sheep, prices)
            .with_grazing_method(GrazingMethod::Maalufah);
            
        let res = stock.with_hawl(true).calculate_zakat(&ZakatConfig::default()).unwrap();
        assert!(!res.is_payable);
        assert_eq!(res.status_reason, Some("Not Sa'imah (naturally grazed)".to_string()));
    }

    #[test]
    fn test_large_number_success() {
        let prices = LivestockPrices::builder()
            .cow_price(dec!(500.0))
            .build().unwrap();

        // 100M + 1 cows. Previously failed due to complexity/iteration limit.
        // Now should pass instantly with O(1) logic.
        
        let stock_large = LivestockAssets::new(100_000_001, LivestockType::Cow, prices);
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
