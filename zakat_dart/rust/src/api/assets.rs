/// Auto-generated asset wrappers for Dart FFI.
///
/// Each asset type is exported using the `dart_export_asset!` macro,
/// providing a consistent fluent builder API that matches the core Rust library.



// ============================================================================
// Business Assets
// ============================================================================

dart_export_asset! {
    /// Business assets (cash, inventory, receivables) for Zakat calculation.
    ///
    /// # Example (Dart)
    /// ```dart
    /// final business = DartBusiness()
    ///     .cash(FrbDecimal.fromString("50000"))
    ///     .inventory(FrbDecimal.fromString("25000"))
    ///     .receivables(FrbDecimal.fromString("10000"))
    ///     .debt(FrbDecimal.fromString("5000"))
    ///     .hawl(true)
    ///     .label("Main Store");
    ///
    /// final result = business.calculate(config);
    /// ```
    zakat::maal::business::BusinessZakat as DartBusiness {
        decimal: [cash, inventory, receivables, debt],
        bool: [hawl],
        string: [label],
    }
}

// ============================================================================
// Precious Metals (Gold & Silver)
// ============================================================================

dart_export_asset! {
    /// Precious metals (gold/silver) asset wrapper for Dart.
    ///
    /// # Example (Dart)
    /// ```dart
    /// final gold = DartPreciousMetals.gold(FrbDecimal.fromString("100"))
    ///     .purity(24)
    ///     .hawl(true)
    ///     .label("Gold Savings");
    ///
    /// final result = gold.calculate(config);
    /// ```
    zakat::maal::precious_metals::PreciousMetals as DartPreciousMetals {
        decimal: [weight, debt],
        bool: [hawl],
        string: [label],
        u32: [purity],
    }
    extra_impl: {
        /// Create a new gold asset with the specified weight in grams.
        #[flutter_rust_bridge::frb(sync)]
        pub fn gold(weight_grams: crate::api::types::FrbDecimal) -> Self {
            Self {
                inner: zakat::maal::precious_metals::PreciousMetals::gold(weight_grams.value),
            }
        }
        
        /// Create a new silver asset with the specified weight in grams.
        #[flutter_rust_bridge::frb(sync)]
        pub fn silver(weight_grams: crate::api::types::FrbDecimal) -> Self {
            Self {
                inner: zakat::maal::precious_metals::PreciousMetals::silver(weight_grams.value),
            }
        }
    }
}

// ============================================================================
// Income Assets
// ============================================================================

dart_export_asset! {
    /// Professional income for Zakat calculation.
    ///
    /// # Example (Dart)
    /// ```dart
    /// final income = DartIncome()
    ///     .income(FrbDecimal.fromString("120000"))
    ///     .expenses(FrbDecimal.fromString("30000"))
    ///     .hawl(true);
    ///
    /// final result = income.calculate(config);
    /// ```
    zakat::maal::income::IncomeZakatCalculator as DartIncome {
        decimal: [income, expenses, debt],
        bool: [hawl],
        string: [label],
    }
}

// ============================================================================
// Investment Assets
// ============================================================================

dart_export_asset! {
    /// Investment assets (stocks, mutual funds) for Zakat calculation.
    ///
    /// # Example (Dart)
    /// ```dart
    /// final investment = DartInvestment()
    ///     .value(FrbDecimal.fromString("100000"))
    ///     .hawl(true);
    ///
    /// final result = investment.calculate(config);
    /// ```
    zakat::maal::investments::InvestmentAssets as DartInvestment {
        decimal: [value, debt],
        bool: [hawl],
        string: [label],
    }
}

// ============================================================================
// Mining Assets
// ============================================================================

dart_export_asset! {
    /// Mining/natural resources for Zakat calculation.
    ///
    /// Note: Mining has different Zakat rates (5-20%) based on extraction method.
    zakat::maal::mining::MiningAssets as DartMining {
        decimal: [value, debt],
        bool: [hawl],
        string: [label],
    }
}
