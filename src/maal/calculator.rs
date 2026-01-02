use rust_decimal::Decimal;
use crate::types::{ZakatDetails, ZakatError, CalculationStep, WealthType};
use crate::math::ZakatDecimal;

/// Parameters required to calculate Zakat for a standard monetary asset.
pub struct MonetaryCalcParams {
    pub total_assets: Decimal,      // Gross assets (e.g., Cash + Inventory, or Market Value)
    pub liabilities: Decimal,       // Deductible liabilities
    pub nisab_threshold: Decimal,   // The threshold to check against
    pub rate: Decimal,              // The Zakat rate (e.g., 0.025)
    pub wealth_type: WealthType,
    pub label: Option<String>,
    pub hawl_satisfied: bool,
    pub trace_steps: Vec<CalculationStep>, // Asset-specific steps leading up to Total Assets
    pub warnings: Vec<String>, // Non-fatal warnings to include in the result
}

/// Standardized Zakat calculation logic for monetary assets.
/// Performs: Hawl Check -> Net Calculation -> Nisab Check -> Rate Application -> Trace Construction.
pub fn calculate_monetary_asset(params: MonetaryCalcParams) -> Result<ZakatDetails, ZakatError> {
    // 1. Hawl Check
    if !params.hawl_satisfied {
        return Ok(ZakatDetails::below_threshold(
            params.nisab_threshold,
            params.wealth_type,
            "Hawl (1 lunar year) not met"
        ).with_label(params.label.unwrap_or_default()));
    }

    // 2. Net Calculation
    // Note: ZakatDecimal handles safe math and context errors
    let net_val = ZakatDecimal::new(params.total_assets)
        .safe_sub(params.liabilities)
        .map_err(|e| e.with_source(params.label.clone().unwrap_or_default()))?;
    
    let net_assets = *net_val;

    // 3. Payability Check
    let is_payable = net_assets >= params.nisab_threshold && net_assets > Decimal::ZERO;
    
    let zakat_due = if is_payable {
        ZakatDecimal::new(net_assets)
            .safe_mul(params.rate)
            .map_err(|e| e.with_source(params.label.clone().unwrap_or_default()))?
            .0
    } else {
        Decimal::ZERO
    };

    // 4. Construct Final Trace
    let mut final_trace = params.trace_steps;
    
    // Auto-add liability step if liabilities exist
    if params.liabilities > Decimal::ZERO {
        final_trace.push(CalculationStep::subtract("step-debts-due-now", "Liabilities", params.liabilities));
    }
    
    final_trace.push(CalculationStep::result("step-net-assets", "Net Assets", net_assets));
    final_trace.push(CalculationStep::compare("step-nisab-check", "Nisab Threshold", params.nisab_threshold));

    if is_payable {
        final_trace.push(CalculationStep::rate("step-rate-applied", "Rate Applied", params.rate));
        final_trace.push(CalculationStep::result("status-due", "Zakat Due", zakat_due));
    } else {
        final_trace.push(CalculationStep::info("status-exempt", "Below Nisab"));
    }

    let mut result = ZakatDetails::with_trace(
        params.total_assets,
        params.liabilities,
        params.nisab_threshold,
        params.rate,
        params.wealth_type,
        final_trace
    ).with_label(params.label.unwrap_or_default());
    
    // Add any warnings from params to the result
    result.warnings.extend(params.warnings);
    
    Ok(result)
}
