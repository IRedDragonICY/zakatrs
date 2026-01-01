import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:zakat/src/rust/frb_generated.dart';
import 'package:zakat/src/rust/api/zakat.dart';
import 'package:integration_test/integration_test.dart';
import 'package:decimal/decimal.dart';
import 'package:zakat/src/extensions.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('Can calculate Business Zakat with ZakatManager', (WidgetTester tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: Text("Test"))));

    // Scenario: Cash 10,000, Gold Price $100/g.
    // 85g Gold = $8,500. Result should be payable.
    // 10,000 * 0.025 = 250.
    
    // Initialize Manager
    final manager = ZakatManager(
      goldPrice: Decimal.parse("100.0").toFrb(),
      silverPrice: Decimal.parse("1.0").toFrb(),
      madhab: "Hanafi",
    );
    
    final result = manager.calculateBusiness(
      cash: Decimal.parse("10000.0").toFrb(),
      inventory: Decimal.zero.toFrb(),
      receivables: Decimal.zero.toFrb(),
      liabilities: Decimal.zero.toFrb(),
    );
    
    print('Debug Business: IsPayable=${result.isPayable}, Due=${result.zakatDue.toDecimal()}, Threshold=${result.nisabThreshold.toDecimal()}');
    
    expect(result.isPayable, true, reason: "Business should be payable");
    expect(result.zakatDue.toDecimal(), Decimal.parse("250.0"));
    
    // Gold Nisab: 85 * 100 = 8500. Silver Nisab: 595 * 1 = 595.
    // Lower is 595.
    expect(result.nisabThreshold.toDecimal(), Decimal.parse("595.0"));
  });

  testWidgets('Can calculate Savings Zakat with ZakatManager', (WidgetTester tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: Text("Test"))));

    // Scenario: Cash 5,000, Gold Price $100/g.
    // Gold Nisab $8500. Silver Nisab $595.
    // Hanafi uses LowerOfTwo (595). 
    // 5000 > 595 -> Payable.
    
    final manager = ZakatManager(
      goldPrice: Decimal.parse("100.0").toFrb(),
      silverPrice: Decimal.parse("1.0").toFrb(),
      madhab: "Hanafi",
    );
    
    final result = manager.calculateSavings(
      cashInHand: Decimal.parse("5000.0").toFrb(),
      bankBalance: Decimal.zero.toFrb(),
    );
    print('Debug Savings: IsPayable=${result.isPayable}, Due=${result.zakatDue.toDecimal()}');

    expect(result.isPayable, true, reason: "Savings should be payable");
    expect(result.zakatDue.toDecimal(), Decimal.parse("125.0")); // 5000 * 0.025
    expect(result.wealthAmount.toDecimal(), Decimal.parse("5000.0"));
  });
  
  testWidgets('Can get Nisab Thresholds', (WidgetTester tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: Text("Test"))));

    final manager = ZakatManager(
      goldPrice: Decimal.parse("100.0").toFrb(),
      silverPrice: Decimal.parse("1.0").toFrb(),
      madhab: "Hanafi",
    );
    
    final thresholds = manager.getNisabThresholds();
    // Tuple handling in Dart/FRB might differ. 
    // Assuming generated code exposes a record or a generic tuple class, OR generated as a class.
    // FRB V2 usually generates standard Dart Records for tuples: (T1, T2) if Dart >= 3.0.
    // If not, it might be a class like `(type, type)`.
    
    // Using $1, $2 if record.
    expect(thresholds.$1.toDecimal(), Decimal.parse("8500.0")); // 100 * 85
    expect(thresholds.$2.toDecimal(), Decimal.parse("595.0")); // 1 * 595
  });
}
