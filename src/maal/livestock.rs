use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::CalculateZakat;

pub enum LivestockType {
    Camel,
    Cow,
    Sheep, // Includes Goats
}

pub struct LivestockAssets {
    pub count: u32,
    pub animal_type: LivestockType,
    pub prices: LivestockPrices,
    pub deductible_liabilities: Decimal,
    pub hawl_satisfied: bool,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LivestockPrices {
    pub sheep_price: Decimal,
    pub cow_price: Decimal, // For Tabi/Musinnah avg or simplified
    pub camel_price: Decimal,
}

impl LivestockPrices {
    pub fn new(
        sheep_price: impl Into<Decimal>,
        cow_price: impl Into<Decimal>,
        camel_price: impl Into<Decimal>,
    ) -> Result<Self, ZakatError> {
        let sheep = sheep_price.into();
        let cow = cow_price.into();
        let camel = camel_price.into();

        if sheep < Decimal::ZERO || cow < Decimal::ZERO || camel < Decimal::ZERO {
            return Err(ZakatError::InvalidInput("Livestock prices must be non-negative".to_string()));
        }

        Ok(Self {
            sheep_price: sheep,
            cow_price: cow,
            camel_price: camel,
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
            deductible_liabilities: Decimal::ZERO,
            hawl_satisfied: true,
            label: None,
        }
    }

    pub fn with_debt(mut self, debt: impl Into<Decimal>) -> Self {
        self.deductible_liabilities = debt.into();
        self
    }

    pub fn with_hawl(mut self, satisfied: bool) -> Self {
        self.hawl_satisfied = satisfied;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl CalculateZakat for LivestockAssets {
    fn calculate_zakat(&self) -> Result<ZakatDetails, ZakatError> {
        if !self.hawl_satisfied {
             // For Livestock, Nisab is count-based, but we need a value for not_payable.
             // We can calculate the value of "Nisab Count" for the type.
             let nisab_count_val = match self.animal_type {
                LivestockType::Sheep => Decimal::from(40) * self.prices.sheep_price,
                LivestockType::Cow => Decimal::from(30) * self.prices.cow_price,
                LivestockType::Camel => Decimal::from(5) * self.prices.camel_price,
             };
             return Ok(ZakatDetails::not_payable(nisab_count_val, crate::types::WealthType::Livestock, "Hawl (1 lunar year) not met")
                .with_label(self.label.clone().unwrap_or_default()));
        }
        // Note: Livestock Zakat is generally strict on counts and tiers, simplified here for "Value" return.
        // We will calculate 'animals due' then multiply by price.
        // Debt deduction is generally not applied to Livestock count directly (Nisab is physical count),
        // but can be applied to final liability or ignore it. 
        // We will IGNORE debts for Livestock count Nisab check as it's physical.

        let (zakat_value, nisab_count) = match self.animal_type {
            LivestockType::Sheep => calculate_sheep_zakat(self.count, self.prices.sheep_price),
            LivestockType::Cow => calculate_cow_zakat(self.count, self.prices.cow_price),
            LivestockType::Camel => calculate_camel_zakat(self.count, &self.prices),
        };

        // We construct ZakatDetails.
        // Total Assets = Count * Price (Approx value of herd)
        let single_price = match self.animal_type {
            LivestockType::Sheep => self.prices.sheep_price,
            LivestockType::Cow => self.prices.cow_price,
            LivestockType::Camel => self.prices.camel_price,
        };
        
        let total_value = Decimal::from(self.count) * single_price;
        let is_payable = zakat_value > Decimal::ZERO;

        Ok(ZakatDetails {
            total_assets: total_value,
            deductible_liabilities: self.deductible_liabilities,
            net_assets: total_value, // Livestock Nisab is on count, not net value usually. If we deduct, we might do it here.
            // But for consistency with args, let's keep it simple. If liabilities were 0 before, they are 0 now.
            // If they are passed, they are just recorded. The "Nisab Check" is count based (done above in separate function).
            // But "Net Assets" might be illustrative. Let's subtract from total_value for reporting.
            // net_assets: total_value - self.deductible_liabilities,
            // Actually, keep logic as preserved. Previous code had deductible_liabilities: Decimal::ZERO.
            // But now we allow self.deductible_liabilities.
            // Let's set deductible_liabilities in the return struct.
            // And net_assets = total_value - deductible_liabilities

            nisab_threshold: Decimal::from(nisab_count) * single_price, 
            is_payable,
            zakat_due: zakat_value,
            wealth_type: crate::types::WealthType::Livestock,
            status_reason: None,
            label: self.label.clone(),
        })
    }
}

fn calculate_sheep_zakat(count: u32, price: Decimal) -> (Decimal, u32) {
    let nisab = 40;
    if count < 40 {
        return (Decimal::ZERO, nisab);
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

    (Decimal::from(sheep_due) * price, nisab)
}

fn calculate_cow_zakat(count: u32, price: Decimal) -> (Decimal, u32) {
    let nisab = 30;
    if count < 30 {
        return (Decimal::ZERO, nisab);
    }

    // Cows Zakat Logic:
    // 30-39: 1 Tabi (Yearling)
    // 40-59: 1 Musinnah (2yo)
    // 60+: Combination of 30s (Tabi) and 40s (Musinnah) to cover the total count.
    
    let mut tabi = 0;
    let mut musinnah = 0;

    if count >= 30 {
        // Algorithm:
        // We iterate downwards to find the combination of 40s (Musinnahs) and 30s (Tabis)
        // that perfectly divides the remainder.
        // We prioritize Musinnahs (40s) as they are generally more valuable, but the primary goal
        // is to cover the count with no remainder if possible.

        let max_m = count / 40;
        let mut best_m = 0;
        let mut best_t = 0;
        let mut found = false;
        
        for m in (0..=max_m).rev() {
            let remainder = count - (m * 40);
            if remainder % 30 == 0 {
                best_m = m;
                best_t = remainder / 30;
                found = true;
                break;
            }
        }
        
        if found {
            musinnah = best_m;
            tabi = best_t;
        } else {
             // Fallback for specific ranges or gaps where exact division isn't possible (e.g., small counts).
             // We apply the standard ranges for < 60 explicitly.
             if count <= 39 { tabi = 1; musinnah = 0; }
             else if count <= 59 { tabi = 0; musinnah = 1; }
             else {
                  // For larger numbers where exact match fails (rare), we default to prioritizing Musinnahs
                  musinnah = count / 40; 
                  let rem = count % 40;
                  if rem >= 30 { tabi += 1; }
             }
        }
    }

    // Value estimation based on pricing ratios relative to a standard cow price:
    // Tabi (1yo) is estimated at 0.7x of standard price.
    // Musinnah (2yo) is estimated at 1.0x of standard price.
    let val_tabi = price * dec!(0.7);
    let val_musinnah = price;
    
    let total_zakat_val = (Decimal::from(tabi) * val_tabi) + (Decimal::from(musinnah) * val_musinnah);
    
    (total_zakat_val, nisab)
}

fn calculate_camel_zakat(count: u32, prices: &LivestockPrices) -> (Decimal, u32) {
    let nisab = 5;
    if count < 5 {
        return (Decimal::ZERO, nisab);
    }
    
    // 5-9: 1 Sheep
    // 10-14: 2 Sheep
    // 15-19: 3 Sheep
    // 20-24: 4 Sheep
    // 25-35: 1 Bint Makhad (Camel 1yo).
    // 36-45: 1 Bint Labun (Camel 2yo).
    // 46-60: 1 Hiqqah (Camel 3yo).
    // 61-75: 1 Jaza'ah (Camel 4yo).
    // 76-90: 2 Bint Labun.
    // 91-120: 2 Hiqqah.
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
        // 1 Bint Labun per 40 camels, 1 Hiqqah per 50 camels.
        // Similar to Cow algothim, we find the combination that maximizes coverage.
        let mut best_h = 0;
        let mut best_b = 0;
        let max_h = count / 50;
        
        for h in (0..=max_h).rev() {
            let rem = count - (h * 50);
            if rem % 40 == 0 {
                best_h = h;
                best_b = rem / 40;
                break;
            }
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
    let v_bm = v_camel * dec!(0.5);
    let v_bl = v_camel * dec!(0.75);
    let v_hq = v_camel;
    let v_jz = v_camel * dec!(1.25);
    
    let total = (Decimal::from(sheep) * v_sheep)
        + (Decimal::from(b_makhad) * v_bm)
        + (Decimal::from(b_labun) * v_bl)
        + (Decimal::from(hiqqah) * v_hq)
        + (Decimal::from(jazaah) * v_jz);
        
    (total, nisab)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheep() {
        let prices = LivestockPrices { sheep_price: dec!(100.0), ..Default::default() };
        let stock = LivestockAssets::new(40, LivestockType::Sheep, prices);
        let res = stock.with_hawl(true).calculate_zakat().unwrap();
        
        // 40 sheep -> 1 sheep due -> $100
        assert_eq!(res.zakat_due, dec!(100.0));
        assert!(res.is_payable);
    }

    #[test]
    fn test_cows_60() {
        let prices = LivestockPrices { cow_price: dec!(1000.0), ..Default::default() };
        let stock = LivestockAssets::new(60, LivestockType::Cow, prices);
        let res = stock.with_hawl(true).calculate_zakat().unwrap();
        
        // 60 -> 2 Tabi.
        // Tabi = 0.7 * 1000 = 700.
        // Total = 1400.
        assert_eq!(res.zakat_due, dec!(1400.0));
    }
}
