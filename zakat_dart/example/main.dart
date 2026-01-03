/// Example demonstrating basic Zakat calculation using the zakat_dart package.
///
/// This example shows how to:
/// 1. Initialize the Rust FFI bridge
/// 2. Configure Zakat calculation with current market prices
/// 3. Calculate Zakat for Business assets
/// 4. Calculate Zakat for Precious Metals (Gold/Silver)
library;

import 'package:zakat/zakat.dart';
import 'package:decimal/decimal.dart';

Future<void> main() async {
  // 1. Initialize the Rust Bridge (required before any calculations)
  await RustLib.init();
  print('✅ Rust bridge initialized successfully.\n');

  // 2. Define Configuration with current market prices
  final config = DartZakatConfig(
    goldPrice: Decimal.parse('85.0').toFrb(),    // $85 per gram of gold
    silverPrice: Decimal.parse('1.0').toFrb(),   // $1 per gram of silver
    madhab: 'hanafi',                            // Hanafi school of thought
  );

  // ─────────────────────────────────────────────────────────────────────────
  // Example 1: Business Zakat Calculation
  // ─────────────────────────────────────────────────────────────────────────
  print('📊 BUSINESS ZAKAT CALCULATION');
  print('─' * 40);

  final business = DartBusiness()
    ..cash(value: Decimal.parse('10000').toFrb())       // Cash on hand
    ..inventory(value: Decimal.parse('5000').toFrb())   // Inventory value
    ..receivables(value: Decimal.parse('2000').toFrb()) // Money owed to you
    ..debt(value: Decimal.parse('1000').toFrb())        // Current liabilities
    ..hawl(value: true);                                 // Held for 1 lunar year

  final businessResult = business.calculate(config: config);
  
  print('   Cash on Hand:      \$10,000');
  print('   Inventory Value:   \$ 5,000');
  print('   Receivables:       \$ 2,000');
  print('   Liabilities:       \$ 1,000');
  print('   ─────────────────────────');
  print('   Net Assets:        \$${businessResult.netAssets.toDecimal()}');
  print('   Nisab Threshold:   \$${businessResult.nisabThreshold.toDecimal()}');
  print('   Zakat Payable?     ${businessResult.isPayable ? "✅ Yes" : "❌ No"}');
  if (businessResult.isPayable) {
    print('   Zakat Due (2.5%):  \$${businessResult.zakatDue.toDecimal()}');
  }
  print('');

  // ─────────────────────────────────────────────────────────────────────────
  // Example 2: Gold Zakat Calculation
  // ─────────────────────────────────────────────────────────────────────────
  print('🥇 GOLD ZAKAT CALCULATION');
  print('─' * 40);

  final gold = DartPreciousMetals.gold(
    weightGrams: Decimal.parse('100').toFrb(),  // 100 grams of gold
  )
    ..purity(value: 22)                          // 22 Karat gold
    ..hawl(satisfied: true);                     // Held for 1 lunar year

  final goldResult = gold.calculate(config: config);
  
  print('   Weight:            100 grams');
  print('   Purity:            22 Karat');
  print('   Market Price:      \$85/gram');
  print('   ─────────────────────────');
  print('   Net Value:         \$${goldResult.netAssets.toDecimal()}');
  print('   Nisab Threshold:   \$${goldResult.nisabThreshold.toDecimal()}');
  print('   Zakat Payable?     ${goldResult.isPayable ? "✅ Yes" : "❌ No"}');
  if (goldResult.isPayable) {
    print('   Zakat Due (2.5%):  \$${goldResult.zakatDue.toDecimal()}');
  }
  print('');

  // ─────────────────────────────────────────────────────────────────────────
  // Example 3: Silver Zakat Calculation (Below Nisab)
  // ─────────────────────────────────────────────────────────────────────────
  print('🥈 SILVER ZAKAT CALCULATION (Below Nisab)');
  print('─' * 40);

  final silver = DartPreciousMetals.silver(
    weightGrams: Decimal.parse('100').toFrb(),  // 100 grams of silver
  )..hawl(satisfied: true);

  final silverResult = silver.calculate(config: config);
  
  print('   Weight:            100 grams');
  print('   Market Price:      \$1/gram');
  print('   ─────────────────────────');
  print('   Net Value:         \$${silverResult.netAssets.toDecimal()}');
  print('   Nisab Threshold:   \$${silverResult.nisabThreshold.toDecimal()}');
  print('   Zakat Payable?     ${silverResult.isPayable ? "✅ Yes" : "❌ No (below nisab)"}');
  print('');

  print('✅ All calculations completed successfully!');
}
