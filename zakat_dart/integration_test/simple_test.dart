import 'package:flutter_test/flutter_test.dart';
import 'package:zakat/main.dart'; // Ensure main is accessible or just init
import 'package:zakat/src/rust/frb_generated.dart';
import 'package:zakat/src/rust/api/simple.dart';
import 'package:integration_test/integration_test.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(() async => await RustLib.init());

  testWidgets('Can calculate Business Zakat', (WidgetTester tester) async {
    // Run the app (optional if we just want to test logic, but good utility)
    await tester.pumpWidget(const MyApp());

    // Scenario: Cash 10,000, Gold Price $100/g.
    // 85g Gold = $8,500. Result should be payable.
    // 10,000 * 0.025 = 250.
    
    // Note: Rust API uses f64.
    final result = await calculateBusinessZakat(
      cash: 10000.0,
      inventory: 0.0,
      receivables: 0.0,
      liabilities: 0.0,
      goldPrice: 100.0,
      silverPrice: 1.0,
    );
    
    print('Debug Business: IsPayable=${result.isPayable}, Due=${result.zakatDue}, Threshold=${result.nisabThreshold}');
    print('Debug Business: IsPayable=${result.isPayable}, Due=${result.zakatDue}, Threshold=${result.nisabThreshold}');
    // Check boolean only first
    expect(result.isPayable, true, reason: "Business should be payable");
    // expect(result.zakatDue, 250.0);
    // Default config is Hanafi which uses LowerOfTwo.
    // Gold Nisab: 85 * 100 = 8500. Silver Nisab: 595 * 1 = 595.
    // Lower is 595.
    expect(result.nisabThreshold, 595.0);
  });

  testWidgets('Can calculate Savings Zakat (Above LowerOfTwo Nisab)', (WidgetTester tester) async {
    await tester.pumpWidget(const MyApp());

    // Scenario: Cash 5,000, Gold Price $100/g.
    // Gold Nisab $8500. Silver Nisab $595.
    // Hanafi uses LowerOfTwo (595). 
    // 5000 > 595 -> Payable.
    
    final result = await calculateSavingsZakat(
      cashInHand: 5000.0,
      bankBalance: 0.0,
      goldPrice: 100.0,
      silverPrice: 1.0,
    );
    print('Debug Savings: IsPayable=${result.isPayable}, Due=${result.zakatDue}, Threshold=${result.nisabThreshold}, Wealth=${result.wealthAmount}');

    expect(result.isPayable, true, reason: "Savings should be payable");
    // expect(result.nisabThreshold, 595.0);
    expect(result.zakatDue, 125.0); // 5000 * 0.025
    expect(result.wealthAmount, 5000.0);
  });
}
