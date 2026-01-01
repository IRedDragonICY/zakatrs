import 'package:flutter_test/flutter_test.dart';
import 'package:zakat/main.dart';
import 'package:zakat/src/rust/frb_generated.dart';
import 'package:zakat/src/rust/api/simple.dart';
import 'package:integration_test/integration_test.dart';
import 'package:decimal/decimal.dart';
import 'package:zakat/src/extensions.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('Can calculate Business Zakat with Decimal', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    // Scenario: Cash 10,000, Gold Price $100/g.
    // 85g Gold = $8,500. Result should be payable.
    // 10,000 * 0.025 = 250.
    
    final result = await calculateBusinessZakat(
      cash: Decimal.parse("10000.0").toFrb(),
      inventory: Decimal.zero.toFrb(),
      receivables: Decimal.zero.toFrb(),
      liabilities: Decimal.zero.toFrb(),
      goldPrice: Decimal.parse("100.0").toFrb(),
      silverPrice: Decimal.parse("1.0").toFrb(),
    );
    
    print('Debug Business: IsPayable=${result.isPayable}, Due=${result.zakatDue.toDecimal()}, Threshold=${result.nisabThreshold.toDecimal()}');
    
    expect(result.isPayable, true, reason: "Business should be payable");
    expect(result.zakatDue.toDecimal(), Decimal.parse("250.0"));
    
    // Gold Nisab: 85 * 100 = 8500. Silver Nisab: 595 * 1 = 595.
    // Lower is 595.
    expect(result.nisabThreshold.toDecimal(), Decimal.parse("595.0"));
  });

  testWidgets('Can calculate Savings Zakat with Decimal', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    // Scenario: Cash 5,000, Gold Price $100/g.
    // Gold Nisab $8500. Silver Nisab $595.
    // Hanafi uses LowerOfTwo (595). 
    // 5000 > 595 -> Payable.
    
    final result = await calculateSavingsZakat(
      cashInHand: Decimal.parse("5000.0").toFrb(),
      bankBalance: Decimal.zero.toFrb(),
      goldPrice: Decimal.parse("100.0").toFrb(),
      silverPrice: Decimal.parse("1.0").toFrb(),
    );
    print('Debug Savings: IsPayable=${result.isPayable}, Due=${result.zakatDue.toDecimal()}');

    expect(result.isPayable, true, reason: "Savings should be payable");
    expect(result.zakatDue.toDecimal(), Decimal.parse("125.0")); // 5000 * 0.025
    expect(result.wealthAmount.toDecimal(), Decimal.parse("5000.0"));
  });
}
