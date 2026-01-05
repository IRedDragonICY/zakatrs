use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use zakat_core::{
    config::ZakatConfig,
    maal::{
        business::BusinessZakat,
        restricted::{RestrictedFund, AccessibilityLevel},
    },
    debt::ReceivableQuality,
    partnership::JointVenture,
    distribution::{BasicNeeds, HouseholdProfile, DistributionCalculator, EligibilityStatus},
    traits::CalculateZakat,
    madhab::Madhab,
};
use zakat_ledger::{MissedZakatCalculator, InflationIndexProvider};
use chrono::Datelike;

// =============================================================================
// Feature 1: Restricted Funds Integration
// =============================================================================
#[test]
fn test_restricted_funds_integration() {
    let mut config = ZakatConfig::default();
    // Shafi (and contemporary) view often supports vesting-based Zakat for pensions
    config = config.with_madhab(Madhab::Shafi);

    // Pension with 100k, 60k vested. Conservative view.
    let fund = RestrictedFund::new_pension(dec!(100000), AccessibilityLevel::LockedUntilRetirement)
        .vested_amount(dec!(60000));
    
    let result = fund.calculate_zakat(&config).unwrap();
    
    assert!(result.is_payable);
    assert_eq!(result.net_assets, dec!(60000));
    assert_eq!(result.zakat_due, dec!(1500)); // 2.5% of 60000
}

// =============================================================================
// Feature 2: Joint Venture Integration
// =============================================================================
#[test]
fn test_joint_venture_shafii() {
    let mut config = ZakatConfig::default();
    config.madhab = Madhab::Shafi; 
    
    // Total 10000 (Above Nisab). Shares 50/50 (5000 each, below Nisab).
    let jv = JointVenture::new(dec!(10000), dec!(0))
        .add_shareholder("A", "Partner A", dec!(0.5))
        .add_shareholder("B", "Partner B", dec!(0.5));
        
    let results = jv.calculate_zakat_distribution(&config).unwrap();
    
    // Both should pay because Total > Nisab
    assert!(results["A"].is_payable);
    assert!(results["B"].is_payable);
    assert_eq!(results["A"].zakat_due, dec!(125)); // 5000 * 0.025
}

// =============================================================================
// Feature 3: Advanced Receivables Integration
// =============================================================================

#[test]
fn test_advanced_receivables_integration() {
    let config = ZakatConfig { gold_price_per_gram: dec!(60), ..Default::default() };
    
    // 5000 Cash + 2000 Good Debt + 10000 Bad Debt
    let business = BusinessZakat::new()
        .cash(dec!(5000))
        .add_receivable("Friend A (Reliable)", dec!(2000), ReceivableQuality::Strong)
        .add_receivable("Friend B (Ghosted)", dec!(10000), ReceivableQuality::Weak)
        .hawl(true);

    let result = business.calculate_zakat(&config).unwrap();

    // Total Net considered = 5000 + 2000 = 7000.
    // Nisab = 85 * 60 = 5100.
    // Net = 7000.
    
    assert!(result.is_payable);
    assert_eq!(result.net_assets, dec!(7000));
}

// =============================================================================
// Feature 4: Retrospective Qada Integration
// =============================================================================
struct MockInflation;
impl InflationIndexProvider for MockInflation {
    fn get_gold_price_at(&self, date: chrono::NaiveDate) -> Option<Decimal> {
         // 2010: $40, 2024: $80
         if date.year() == 2010 { Some(dec!(40)) } else { Some(dec!(80)) }
    }
    fn get_cpi_at(&self, _: chrono::NaiveDate) -> Option<Decimal> { None }
}

#[test]
fn test_qada_integration() {
    let provider = MockInflation;
    let calculator = MissedZakatCalculator::new(&provider);
    
    let due = chrono::NaiveDate::from_ymd_opt(2010, 1, 1).unwrap();
    let pay = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    
    let res = calculator.calculate_gold_standard(dec!(1000), due, pay).unwrap();
    
    // 1000 / 40 = 25g. 25g * 80 = 2000.
    assert_eq!(res.adjusted_amount_to_pay, dec!(2000));
}

// =============================================================================
// Feature 5: Distribution Integration
// =============================================================================
#[test]
fn test_distribution_integration() {
    let needs = BasicNeeds {
        food: dec!(500),
        shelter: dec!(1000), 
        utilities: dec!(800), // Boost total for test expectation
        ..Default::default() 
    };
    // Total Needs approx 2200
    
    // Income 1000.
    let profile = HouseholdProfile::new(dec!(1000), 0);
    
    let status = DistributionCalculator::check_eligibility(&needs, &profile);
    // 1000 / 2200 = 0.45 (< 0.5)
    assert_eq!(status, EligibilityStatus::Fakir);
}
