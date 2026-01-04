use colored::Colorize;
use inquire::{Confirm, CustomType, Select, Text};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use zakat_core::prelude::*;

/// Runs the guided Zakat wizard.
pub fn run_wizard_mode() -> Result<ZakatPortfolio, Box<dyn std::error::Error>> {
    let mut portfolio = ZakatPortfolio::new();
    
    println!("\n{}", "🧙 GUIDED ZAKAT WIZARD 🧙".bright_cyan().bold());
    println!("{}", "This wizard will guide you through adding your assets step-by-step.".dimmed());
    println!("{}", "You can press Ctrl+C at any time to exit.".dimmed());
    println!();

    // Helper validator for non-negative Decimals
    let non_negative = |input: &Decimal| {
        if *input < Decimal::ZERO {
            Ok(inquire::validator::Validation::Invalid(inquire::validator::ErrorMessage::Custom("Value must be non-negative".to_string())))
        } else {
            Ok(inquire::validator::Validation::Valid)
        }
    };

    // 1. Precious Metals
    if Confirm::new("Do you possess Gold or Silver (jewelry, bars, coins)?")
        .with_default(false)
        .with_help_message("Zakat is due on gold/silver if it reaches the Nisab threshold.")
        .prompt()? 
    {
        println!("\n{}", "--- Precious Metals ---".bright_yellow());
        
        // Gold loop
        loop {
            if !Confirm::new("Add a Gold item?").with_default(true).prompt()? { break; }
            
            let weight: Decimal = CustomType::new("Weight (grams):")
                .with_placeholder("e.g. 85.0")
                .with_error_message("Please enter a valid number")
                .with_validator(non_negative)
                .prompt()?;
                
            let purity = Select::new("Purity (Karat):", vec![
                "24K", "22K", "21K", "18K", "Other"
            ]).prompt()?;
            
            let purity_val = match purity {
                "24K" => dec!(1.0),
                "22K" => dec!(0.916),
                "21K" => dec!(0.875),
                "18K" => dec!(0.750),
                "Other" => {
                    let p: Decimal = CustomType::new("Enter purity (0.0 - 1.0):")
                        .with_help_message("e.g. 0.875 for 21K")
                        .with_validator(non_negative)
                        .with_validator(|p: &Decimal| {
                            if *p > dec!(1.0) {
                                Ok(inquire::validator::Validation::Invalid(inquire::validator::ErrorMessage::Custom("Purity cannot exceed 1.0".to_string())))
                            } else {
                                Ok(inquire::validator::Validation::Valid)
                            }
                        })
                        .prompt()?;
                    p
                },
                _ => dec!(1.0),
            };
            

            
            let final_weight = weight * purity_val;
            
            let mut asset = PreciousMetals::new();
            asset.weight_grams = final_weight;
            asset.metal_type = Some(WealthType::Gold);
            
             portfolio = portfolio.add(asset);
             println!("{}", "Added Gold item.".green());
        }

        // Silver loop
        loop {
            if !Confirm::new("Add a Silver item?").with_default(false).prompt()? { break; }
             let weight: Decimal = CustomType::new("Weight (grams):").with_validator(non_negative).prompt()?;
             let mut asset = PreciousMetals::new();
             asset.weight_grams = weight;
             asset.metal_type = Some(WealthType::Silver);
             
             portfolio = portfolio.add(asset);
             println!("{}", "Added Silver item.".green());
        }
    }
    
    // 2. Cash / Savings
    if Confirm::new("Do you have cash savings, bank accounts, or cash on hand?")
        .with_default(false)
        .prompt()? 
    {
        println!("\n{}", "--- Cash & Savings ---".bright_green());
        loop {
             if !Confirm::new("Add a Cash entry?").with_default(true).prompt()? { break; }
             
             let amount: Decimal = CustomType::new("Amount:")
                 .with_validator(non_negative)
                 .prompt()?;
             let label: String = Text::new("Description:").with_default("Savings").prompt()?;
             
             let mut asset = BusinessZakat::new();
             asset.cash_on_hand = amount;
             let asset = asset.label(label);
             
             portfolio = portfolio.add(asset);
        }
    }
    
    // 3. Business Assets
    if Confirm::new("Do you own a business or trade goods?")
        .with_default(false)
        .prompt()? 
    {
        println!("\n{}", "--- Business Assets ---".bright_blue());
        
        let cash_on_hand: Decimal = CustomType::new("Business Cash on Hand:").with_default(dec!(0)).with_validator(non_negative).prompt()?;
        let inventory: Decimal = CustomType::new("Value of Inventory (Goods for Sale):").with_default(dec!(0)).with_validator(non_negative).prompt()?;
        let receivables: Decimal = CustomType::new("Money Owed TO You (Good Debt):").with_default(dec!(0)).with_validator(non_negative).prompt()?;
        let debts: Decimal = CustomType::new("Debts/Expenses Due NOW:").with_default(dec!(0)).with_validator(non_negative).prompt()?;
        
        let mut asset = BusinessZakat::new();
        asset.cash_on_hand = cash_on_hand;
        asset.inventory_value = inventory;
        asset.receivables = receivables;
        
        let asset = asset
            .add_liability("Short-term Debt", debts)
            .label("Business Assets"); 
            
             // Note: debt() method might not be generated if named_liabilities replace it?
             // But macro deprecates liabilities_due_now. `add_liability` logic should be used or `debt` wrapper.
             // Looking at macros.rs line 98, `debt` setter exists and sets `liabilities_due_now`.
             // But my wizard logic above used `debts`. I'll use `add_liability`.
            
        portfolio = portfolio.add(asset);
    }
    
    // 4. Investments
    if Confirm::new("Do you have stock market investments, mutual funds, or crypto?")
        .with_default(false)
        .prompt()? 
    {
         println!("\n{}", "--- Investments ---".bright_magenta());
         // Simple prompt for total value for now
         let value: Decimal = CustomType::new("Total Market Value of Investments:").with_validator(non_negative).prompt()?;
         let mut asset = InvestmentAssets::new();
         asset.value = value;
         let asset = asset.label("Investments");
         portfolio = portfolio.add(asset);
    }

    println!("\n{}", "✅ Wizard complete! Calculating...".bold());
    
    Ok(portfolio)
}
