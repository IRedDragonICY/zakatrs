use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use zakat::{ZakatConfig, CalculateZakat, WealthType};
use zakat::maal::business::{BusinessAssets, BusinessZakatCalculator};
use zakat::maal::income::{IncomeZakatCalculator, IncomeCalculationMethod};
use zakat::maal::investments::{InvestmentAssets, InvestmentType};
use zakat::maal::precious_metals::PreciousMetal;
use zakat::maal::agriculture::{AgricultureAssets, IrrigationMethod};
use zakat::maal::livestock::{LivestockAssets, LivestockType, LivestockPrices};
use zakat::maal::mining::{MiningAssets, MiningType};
use zakat::fitrah::calculate_fitrah;

// Helper to print results consistently
fn print_case(title: &str, result: Result<zakat::ZakatDetails, zakat::ZakatError>, expected_payable: bool) {
    println!("\n=== {} ===", title);
    match result {
        Ok(details) => {
            println!("Wealth Type: {:?}", details.wealth_type);
            println!("Net Assets : ${}", details.net_assets);
            println!("Nisab      : ${}", details.nisab_threshold);
            println!("Payable?   : {}", if details.is_payable { "YES" } else { "NO" });
            if details.is_payable {
                println!("ZAKAT DUE  : ${}", details.zakat_due);
            }
            // Basic assertion for the example runner
            if expected_payable != details.is_payable {
                println!("! WARNING: Expected payable status matching {} failed !", expected_payable);
            }
        },
        Err(e) => println!("Error: {}", e),
    }
}

fn main() {
    let config = ZakatConfig::new(dec!(65.0), dec!(1.0)); // Gold $65/g
    println!("Global Config: Gold Price ${}/g", config.gold_price_per_gram);

    // CASE 1: The Freelancer (Professional Income)
    // Earned $4000 this month. Expenses $1500. Net $2500.
    // Gold Price $65 -> Nisab 85g = $5525.
    // Net Income < Nisab. Not Payable (if calculated monthly strictly on surplus).
    // Note: Some opinions accumulate annual income. Assuming monthly calculation here.
    let freelancer = IncomeZakatCalculator::new(
        dec!(4000.0), dec!(1500.0), IncomeCalculationMethod::Net, &config
    ).unwrap();
    print_case("Case 1: Freelancer (Net Income)", freelancer.calculate_zakat(None), false);

    // CASE 2: The Startup Founder (Business Assets - Equity)
    // Cash: $500k. Inventory/IP Valued(?): $0. Short Debt: $50k.
    // Liquid Assets for Zakat: $500k. Debt: $50k. Net: $450k.
    // Nisab ~$5.5k. Payable.
    let startup = BusinessAssets::new(dec!(500000.0), dec!(0.0), dec!(0.0), dec!(50000.0));
    let startup_calc = BusinessZakatCalculator::new(startup, &config).unwrap();
    print_case("Case 2: Startup Founder (Business Cash)", startup_calc.calculate_zakat(None), true);

    // CASE 3: The Gold Saver (Precious Metals)
    // Has 150g Gold bars.
    // Nisab 85g. Payable.
    let saver = PreciousMetal::new(dec!(150.0), WealthType::Gold, &config).unwrap();
    print_case("Case 3: Gold Saver (150g)", saver.calculate_zakat(None), true);

    // CASE 4: The Crypto Trader (Investments)
    // Portfolio worth $3000.
    // Nisab $5525. Not Payable.
    let crypto = InvestmentAssets::new(dec!(3000.0), InvestmentType::Crypto, &config).unwrap();
    print_case("Case 4: Crypto Trader (Small Portfolio)", crypto.calculate_zakat(None), false);

    // CASE 5: The Rice Farmer (Agriculture - Rain Fed)
    // Harvested 1000kg Rice. No debt.
    // Nisab 653kg. Config default used.
    // Rate 10% (Rain).
    // Price per kg: $0.50 (Locally).
    // Value: $500. Zakat: 10% = $50.
    let farmer_rain = AgricultureAssets::new(
        dec!(1000.0), dec!(0.50), IrrigationMethod::Rain, &config
    ).unwrap();
    print_case("Case 5: Rice Farmer (Rain Fed)", farmer_rain.calculate_zakat(None), true);

    // CASE 6: The Modern Farmer (Agriculture - Irrigated/Costly)
    // Harvested 1000kg.
    // Rate 5%. Zakat: $25.
    let farmer_irr = AgricultureAssets::new(
        dec!(1000.0), dec!(0.50), IrrigationMethod::Irrigated, &config
    ).unwrap();
    print_case("Case 6: Modern Farmer (Irrigated)", farmer_irr.calculate_zakat(None), true);

    // CASE 7: The Sheep Herder (Livestock)
    // 50 Sheep.
    // Nisab 40. Payable.
    // Rate: 1 Sheep (40-120 range).
    // Sheep Price: $150.
    // Due: $150.
    let livestock_prices = LivestockPrices { sheep_price: dec!(150.0), ..Default::default() };
    let shepherd = LivestockAssets::new(50, LivestockType::Sheep, livestock_prices);
    print_case("Case 7: Sheep Herder (50 Sheep)", shepherd.calculate_zakat(None), true);

    // CASE 8: The Treasure Hunter (Rikaz)
    // Found ancient coins worth $10,000.
    // Rate 20%. No Nisab check strictly (or minimal).
    // Due: $2,000.
    let treasure = MiningAssets::new(dec!(10000.0), MiningType::Rikaz, &config).unwrap();
    print_case("Case 8: Treasure Hunter (Rikaz)", treasure.calculate_zakat(None), true);

    // CASE 9: The Stock Investor (Long Term)
    // Stocks worth $50,000.
    // Conservative opinion: 2.5% on Market Value for liquid stocks.
    // Due: $1,250.
    let stocks = InvestmentAssets::new(dec!(50000.0), InvestmentType::Stock, &config).unwrap();
    print_case("Case 9: Stock Investor (Market Value)", stocks.calculate_zakat(None), true);

    // CASE 10: Zakat Fitrah for Family
    // Family of 5.
    // Rice Price $1.50/kg.
    // 2.5kg per person.
    // Total kg: 12.5kg.
    // Total Value: 12.5 * 1.5 = $18.75.
    let fitrah_res = calculate_fitrah(5, dec!(1.50), None); // Default 2.5kg
    print_case("Case 10: Family Fitrah (5 People)", fitrah_res, true);
}
