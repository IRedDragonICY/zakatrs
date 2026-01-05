//! # Fiqh Compliance: Joint Venture & Mixture of Assets (Khultat)
//!
//! Handles "Dam' al-Amwal" for partnerships.
//!
//! ## Fiqh Rulings
//! 1. **Shafi'i (Khultat)**: 
//!    - Assets of the partnership are treated as a single entity.
//!    - Nisab is checked against the *total mixed assets*, not individual shares.
//!    - Zakat is calculated on the total, then distributed proportionally.
//! 2. **Hanafi / Maliki**:
//!    - Partnerships are just collections of individual ownerships.
//!    - Nisab is checked against each partner's share individually.
//!    - If a partner's share < Nisab, they pay nothing (even if total > Nisab).

use rust_decimal::Decimal;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::types::{ZakatDetails, ZakatError};
use crate::traits::{ZakatConfigArgument};

/// A shareholder in the joint venture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Shareholder {
    pub id: String, // Uuid or string ID
    pub name: String,
    pub ownership_percentage: Decimal, // 0.0 to 1.0
}

/// Type of Ownership/Partnership structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartnershipType {
    /// **Sole Proprietorship**: Single owner. Normal rules apply.
    SoleProprietorship,
    
    /// **Syirkah / Khultat**: Joint Venture / Partnership.
    /// Subject to different rulings based on Madhab (Shafi'i vs others).
    Syirkah,
}

/// Represents a Joint Venture Portfolio.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JointVenture {
    pub total_assets: Decimal, 
    pub total_liabilities: Decimal,
    pub shareholders: Vec<Shareholder>,
    pub partnership_type: PartnershipType,
}

impl JointVenture {
    pub fn new(total_assets: Decimal, total_liabilities: Decimal) -> Self {
        Self {
            total_assets,
            total_liabilities,
            shareholders: Vec::new(),
            partnership_type: PartnershipType::Syirkah,
        }
    }

    pub fn add_shareholder(mut self, id: &str, name: &str, percentage: Decimal) -> Self {
        self.shareholders.push(Shareholder {
            id: id.to_string(),
            name: name.to_string(),
            ownership_percentage: percentage,
        });
        self
    }
    
    /// Calculates net assets for the entire entity.
    pub fn net_assets(&self) -> Decimal {
        self.total_assets - self.total_liabilities
    }

    /// Calculates Zakat for each partner based on the configuration strategy.
    /// Returns a map of Shareholder ID -> ZakatDetails.
    pub fn calculate_zakat_distribution<C: ZakatConfigArgument>(&self, config: C) -> Result<HashMap<String, ZakatDetails>, ZakatError> {
        let config_cow = config.resolve_config();
        let config = config_cow.as_ref();
        
        let net_assets = self.net_assets();
        let nisab_threshold = config.get_monetary_nisab_threshold();
        // Since trade_goods_rate is a Decimal (e.g., 0.025), not a struct, we use it directly.
        let zakat_rate = config.strategy.get_rules().trade_goods_rate; 
        
        let mut distribution = HashMap::new();
        
        // Check if we apply Shafi'i Khultat logic (treat as single entity)
        let use_khultat_entity_rule = matches!(config.madhab, crate::madhab::Madhab::Shafi);

        if use_khultat_entity_rule && self.partnership_type == PartnershipType::Syirkah {
            // SHAFI'I: Check Nisab on TOTAL assets
            if net_assets >= nisab_threshold {
                // Total is zakatable. Distribute burden proportionally.
                let total_zakat = net_assets * zakat_rate;
                
                for shareholder in &self.shareholders {
                    let share_net = net_assets * shareholder.ownership_percentage;
                    let share_zakat = total_zakat * shareholder.ownership_percentage;
                    
                    let mut details = ZakatDetails::new(
                        share_net, // total assets same as net here roughly
                        Decimal::ZERO,
                        nisab_threshold,
                        zakat_rate,
                        crate::types::WealthType::Business
                    );
                    details.zakat_due = share_zakat;
                    details.is_payable = true;
                    details.notes.push("Calculated using Shafi'i Khultat (Joint) rules. Nisab checked against total entity.".to_string());
                    
                    distribution.insert(shareholder.id.clone(), details);
                }
            } else {
                // Total < Nisab. No one pays.
                for shareholder in &self.shareholders {
                    let share_net = net_assets * shareholder.ownership_percentage;
                    let mut details = ZakatDetails::new(
                        share_net,
                        Decimal::ZERO,
                        nisab_threshold,
                        zakat_rate,
                        crate::types::WealthType::Business
                    );
                    details.is_payable = false;
                    details.zakat_due = Decimal::ZERO;
                    details.notes.push("Total entity assets below Nisab (Shafi'i view).".to_string());
                    
                    distribution.insert(shareholder.id.clone(), details);
                }
            }
        } else {
            // HANAFI / MALIKI / STANDARD: Check Nisab on INDIVIDUAL share
            for shareholder in &self.shareholders {
                let share_net = net_assets * shareholder.ownership_percentage;
                
                let (zakat_due, is_payable, note) = if share_net >= nisab_threshold {
                    (share_net * zakat_rate, true, "Share exceeds Nisab (Individual Check).".to_string())
                } else {
                    (Decimal::ZERO, false, "Share below Nisab (Individual Check).".to_string())
                };

                let mut details = ZakatDetails::new(
                    share_net,
                    Decimal::ZERO,
                    nisab_threshold,
                    zakat_rate,
                    crate::types::WealthType::Business
                );
                details.zakat_due = zakat_due;
                details.is_payable = is_payable;
                details.notes.push(note);
                
                distribution.insert(shareholder.id.clone(), details);
            }
        }

        Ok(distribution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ZakatConfig;
    use crate::madhab::Madhab;
    use rust_decimal_macros::dec;

    #[test]
    fn test_shafii_khultat_mixed_nisab() {
        let mut config = ZakatConfig { gold_price_per_gram: Decimal::from(100), ..Default::default() };
        config.madhab = Madhab::Shafi; 
        
        // Total 10000 > 8500. Shares 0.4=4000, 0.6=6000 (Both < 8500)
        let jv = JointVenture::new(dec!(10000), dec!(0))
            .add_shareholder("A", "Partner A", dec!(0.4))
            .add_shareholder("B", "Partner B", dec!(0.6));
            
        let results = jv.calculate_zakat_distribution(&config).unwrap();
        
        let res_a = results.get("A").unwrap();
        assert!(res_a.is_payable);
        assert_eq!(res_a.zakat_due, dec!(4000) * dec!(0.025)); // 100
        
        let res_b = results.get("B").unwrap();
        assert!(res_b.is_payable);
    }

    #[test]
    fn test_hanafi_individual_nisab() {
        let mut config = ZakatConfig { gold_price_per_gram: Decimal::from(100), ..Default::default() };
        config.madhab = Madhab::Hanafi; 
        
        let jv = JointVenture::new(dec!(10000), dec!(0))
            .add_shareholder("A", "Partner A", dec!(0.4))
            .add_shareholder("B", "Partner B", dec!(0.6));
            
        let results = jv.calculate_zakat_distribution(&config).unwrap();
        
        assert!(!results.get("A").unwrap().is_payable);
        assert!(!results.get("B").unwrap().is_payable);
    }
}
