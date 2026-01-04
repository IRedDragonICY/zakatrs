import 'package:flutter_test/flutter_test.dart';
import 'package:zakat/zakat.dart';
import 'package:decimal/decimal.dart';

void main() {
  setUpAll(() async {
    await RustLib.init();
  });

  test('Business Zakat Calculation via Macro Wrapper', () async {
    // 1. Setup Config
    // Gold Price = 100/g. Nisab Gold = 85g * 100 = 8500.
    final config = DartZakatConfig(
      goldPrice: Decimal.parse('100').toFrb(),
      silverPrice: Decimal.parse('2').toFrb(),
      madhab: 'hanafi',
    );

    // 2. Create Business Asset
    // Cash: 5000
    // Inventory: 4000
    // Total: 9000.
    // Net: 9000 > 8500 (Nisab). Payable.
    // Zakat: 2.5% * 9000 = 225.
    final business = DartBusiness()
      ..cash(value: Decimal.parse('5000').toFrb())
      ..inventory(value: Decimal.parse('4000').toFrb())
      ..hawl(value: true);

    // 3. Calculate
    final result = await business.calculate(config: config);

    // 4. Verify
    print("Zakat Due: ${result.zakatDue.toDecimal()}");
    expect(result.isPayable, true);
    expect(result.zakatDue.toDecimal(), Decimal.parse('225.0'));
    expect(result.wealthType, 'Business');
  });

  test('Precious Metals (Gold) Calculation', () async {
    // 1. Setup Config
    final config = DartZakatConfig(
      goldPrice: Decimal.parse('100').toFrb(),
      silverPrice: Decimal.parse('2').toFrb(),
      madhab: 'hanafi',
    );

    // 2. Create Gold Asset
    // 100g > 85g Nisab.
    // Zakat: 2.5% * 100g * 100/g = 250.
    final gold =
        DartPreciousMetals.gold(weightGrams: Decimal.parse('100').toFrb())
          ..purity(value: 24)
          ..hawl(
            satisfied: true,
          ); // Generated param name is 'satisfied' for hawl in precious_metals macro

    // 3. Calculate
    final result = await gold.calculate(config: config);

    // 4. Verify
    expect(result.isPayable, true);
    expect(result.zakatDue.toDecimal(), Decimal.parse('250.0'));
    expect(result.wealthType, contains('Gold'));
  });
}
