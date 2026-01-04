/// Compliance Test Runner for zakat_dart Flutter Bindings.
///
/// This test runner loads the golden data from `zakat_suite.json` and verifies
/// that the Dart bindings produce identical results to the Rust core.
///
/// Usage:
///   flutter test test/compliance_test.dart
library;

import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:zakat/zakat.dart';
import 'package:decimal/decimal.dart';

/// Test case structure from the JSON schema.
class TestCase {
  final String id;
  final String description;
  final String category;
  final String assetType;
  final Map<String, dynamic> config;
  final Map<String, dynamic> input;
  final Map<String, dynamic> expected;

  TestCase({
    required this.id,
    required this.description,
    required this.category,
    required this.assetType,
    required this.config,
    required this.input,
    required this.expected,
  });

  factory TestCase.fromJson(Map<String, dynamic> json) {
    return TestCase(
      id: json['id'] as String,
      description: json['description'] as String,
      category: json['category'] as String,
      assetType: json['asset_type'] as String,
      config: json['config'] as Map<String, dynamic>,
      input: json['input'] as Map<String, dynamic>,
      expected: json['expected'] as Map<String, dynamic>,
    );
  }
}

/// Loads the compliance test suite from JSON.
Future<List<TestCase>> loadComplianceSuite() async {
  // Try multiple paths for the fixture file
  final paths = [
    '../tests/fixtures/zakat_suite.json',
    'tests/fixtures/zakat_suite.json',
    '../../tests/fixtures/zakat_suite.json',
  ];

  for (final path in paths) {
    final file = File(path);
    if (await file.exists()) {
      final content = await file.readAsString();
      final json = jsonDecode(content) as Map<String, dynamic>;
      final cases = (json['cases'] as List<dynamic>)
          .map((c) => TestCase.fromJson(c as Map<String, dynamic>))
          .toList();
      return cases;
    }
  }

  // Gracefully skip when fixture not found (needed for pub.dev verification)
  // ignore: avoid_print
  print('⚠️ WARNING: zakat_suite.json not found. Skipping compliance tests.');
  // ignore: avoid_print
  print('   To generate: cargo run -p zakat-test-gen');
  return [];
}

/// Creates a DartZakatConfig from test case config.
DartZakatConfig createConfig(Map<String, dynamic> config) {
  final goldPrice = config['gold_price_per_gram'] as String;
  final silverPrice = config['silver_price_per_gram'] as String;
  final madhab = config['madhab'] as String? ?? 'hanafi';

  return DartZakatConfig(
    goldPrice: Decimal.parse(goldPrice).toFrb(),
    silverPrice: Decimal.parse(silverPrice).toFrb(),
    madhab: madhab,
  );
}

/// Compares two decimal strings with tolerance.
void expectDecimalEqual(String actual, String expected, String message) {
  final actualDec = double.tryParse(actual) ?? 0.0;
  final expectedDec = double.tryParse(expected) ?? 0.0;
  final diff = (actualDec - expectedDec).abs();

  expect(
    diff,
    lessThan(0.0000001),
    reason: '$message\nExpected: $expected\nActual: $actual',
  );
}

void main() {
  late List<TestCase> allCases;

  setUpAll(() async {
    // Initialize the Rust library
    await RustLib.init();

    // Load test cases
    allCases = await loadComplianceSuite();
    // ignore: avoid_print
    print('📋 Loaded ${allCases.length} test cases');
  });

  group('Business Compliance Tests', () {
    test('Run all business cases', () async {
      final businessCases = allCases
          .where((c) => c.assetType == 'business')
          .toList();
      // ignore: avoid_print
      print('\n🏢 Testing ${businessCases.length} business cases...');

      for (final testCase in businessCases) {
        // Skip error cases for now (validation tests)
        if (testCase.expected['error_code'] != null) {
          continue;
        }

        final config = createConfig(testCase.config);

        // Get field values from input (flat structure, not nested in 'fields')
        final cashOnHand = testCase.input['cash_on_hand'] as String? ?? '0';
        final inventoryValue =
            testCase.input['inventory_value'] as String? ?? '0';
        final receivables = testCase.input['receivables'] as String? ?? '0';
        final liabilities =
            testCase.input['liabilities_due_now'] as String? ?? '0';
        final hawlSatisfied = testCase.input['hawl_satisfied'] as bool? ?? true;

        // Use fluent builder API
        final business = DartBusiness()
          ..cash(value: Decimal.parse(cashOnHand).toFrb())
          ..inventory(value: Decimal.parse(inventoryValue).toFrb())
          ..receivables(value: Decimal.parse(receivables).toFrb())
          ..debt(value: Decimal.parse(liabilities).toFrb())
          ..hawl(value: hawlSatisfied);

        final result = business.calculate(config: config);

        expect(
          result.isPayable,
          testCase.expected['is_payable'],
          reason:
              '[${testCase.id}] ${testCase.description} - is_payable mismatch',
        );

        expectDecimalEqual(
          result.zakatDue.toDecimal().toString(),
          testCase.expected['zakat_due'] as String,
          '[${testCase.id}] ${testCase.description} - zakat_due mismatch',
        );

        expectDecimalEqual(
          result.netAssets.toDecimal().toString(),
          testCase.expected['net_assets'] as String,
          '[${testCase.id}] ${testCase.description} - net_assets mismatch',
        );

        // ignore: avoid_print
        print('  ✅ ${testCase.id}: ${testCase.description}');
      }
    });
  });

  group('Gold Compliance Tests', () {
    test('Run all gold cases', () async {
      final goldCases = allCases.where((c) => c.assetType == 'gold').toList();
      // ignore: avoid_print
      print('\n🥇 Testing ${goldCases.length} gold cases...');

      for (final testCase in goldCases) {
        // Skip error cases for now
        if (testCase.expected['error_code'] != null) {
          continue;
        }

        // Skip Shafi madhab tests (Python does this too)
        final madhab = testCase.config['madhab'] as String? ?? 'hanafi';
        if (madhab != 'hanafi') {
          // ignore: avoid_print
          print('  ⏭️ ${testCase.id}: Skipped (${madhab} madhab)');
          continue;
        }

        final config = createConfig(testCase.config);

        final weightGrams = testCase.input['weight_grams'] as String? ?? '0';
        final purity =
            int.tryParse(testCase.input['purity'] as String? ?? '24') ?? 24;
        final liabilities =
            testCase.input['liabilities_due_now'] as String? ?? '0';
        final hawlSatisfied = testCase.input['hawl_satisfied'] as bool? ?? true;

        // Use factory constructor and fluent API
        final gold =
            DartPreciousMetals.gold(
                weightGrams: Decimal.parse(weightGrams).toFrb(),
              )
              ..purity(value: purity)
              ..debt(amount: Decimal.parse(liabilities).toFrb())
              ..hawl(satisfied: hawlSatisfied);

        final result = gold.calculate(config: config);

        expect(
          result.isPayable,
          testCase.expected['is_payable'],
          reason:
              '[${testCase.id}] ${testCase.description} - is_payable mismatch',
        );

        expectDecimalEqual(
          result.zakatDue.toDecimal().toString(),
          testCase.expected['zakat_due'] as String,
          '[${testCase.id}] ${testCase.description} - zakat_due mismatch',
        );

        // ignore: avoid_print
        print('  ✅ ${testCase.id}: ${testCase.description}');
      }
    });
  });

  group('Silver Compliance Tests', () {
    test('Run all silver cases', () async {
      final silverCases = allCases
          .where((c) => c.assetType == 'silver')
          .toList();
      // ignore: avoid_print
      print('\n🥈 Testing ${silverCases.length} silver cases...');

      for (final testCase in silverCases) {
        // Skip error cases
        if (testCase.expected['error_code'] != null) {
          continue;
        }

        final config = createConfig(testCase.config);

        final weightGrams = testCase.input['weight_grams'] as String? ?? '0';
        final purity =
            int.tryParse(testCase.input['purity'] as String? ?? '1000') ?? 1000;
        final liabilities =
            testCase.input['liabilities_due_now'] as String? ?? '0';
        final hawlSatisfied = testCase.input['hawl_satisfied'] as bool? ?? true;

        // Use factory constructor and fluent API
        final silver =
            DartPreciousMetals.silver(
                weightGrams: Decimal.parse(weightGrams).toFrb(),
              )
              ..purity(value: purity)
              ..debt(amount: Decimal.parse(liabilities).toFrb())
              ..hawl(satisfied: hawlSatisfied);

        final result = silver.calculate(config: config);

        expect(
          result.isPayable,
          testCase.expected['is_payable'],
          reason:
              '[${testCase.id}] ${testCase.description} - is_payable mismatch',
        );

        expectDecimalEqual(
          result.zakatDue.toDecimal().toString(),
          testCase.expected['zakat_due'] as String,
          '[${testCase.id}] ${testCase.description} - zakat_due mismatch',
        );

        // ignore: avoid_print
        print('  ✅ ${testCase.id}: ${testCase.description}');
      }
    });
  });

  group('Edge Cases Compliance Tests', () {
    test('Run all edge cases', () async {
      final edgeCases = allCases
          .where((c) => c.category == 'edge_case')
          .toList();
      // ignore: avoid_print
      print('\n🔬 Testing ${edgeCases.length} edge cases...');

      for (final testCase in edgeCases) {
        // Skip error cases
        if (testCase.expected['error_code'] != null) {
          continue;
        }

        final config = createConfig(testCase.config);
        DartZakatResult? result;

        if (testCase.assetType == 'business') {
          final cashOnHand = testCase.input['cash_on_hand'] as String? ?? '0';
          final inventoryValue =
              testCase.input['inventory_value'] as String? ?? '0';
          final receivables = testCase.input['receivables'] as String? ?? '0';
          final liabilities =
              testCase.input['liabilities_due_now'] as String? ?? '0';
          final hawlSatisfied =
              testCase.input['hawl_satisfied'] as bool? ?? true;

          final business = DartBusiness()
            ..cash(value: Decimal.parse(cashOnHand).toFrb())
            ..inventory(value: Decimal.parse(inventoryValue).toFrb())
            ..receivables(value: Decimal.parse(receivables).toFrb())
            ..debt(value: Decimal.parse(liabilities).toFrb())
            ..hawl(value: hawlSatisfied);

          result = business.calculate(config: config);
        } else if (testCase.assetType == 'gold') {
          final weightGrams = testCase.input['weight_grams'] as String? ?? '0';
          final purity =
              int.tryParse(testCase.input['purity'] as String? ?? '24') ?? 24;
          final liabilities =
              testCase.input['liabilities_due_now'] as String? ?? '0';
          final hawlSatisfied =
              testCase.input['hawl_satisfied'] as bool? ?? true;

          final gold =
              DartPreciousMetals.gold(
                  weightGrams: Decimal.parse(weightGrams).toFrb(),
                )
                ..purity(value: purity)
                ..debt(amount: Decimal.parse(liabilities).toFrb())
                ..hawl(satisfied: hawlSatisfied);

          result = gold.calculate(config: config);
        }

        if (result != null) {
          expect(
            result.isPayable,
            testCase.expected['is_payable'],
            reason:
                '[${testCase.id}] ${testCase.description} - is_payable mismatch',
          );

          // ignore: avoid_print
          print('  ✅ ${testCase.id}: ${testCase.description}');
        }
      }
    });
  });

  group('Configuration Compliance Tests', () {
    test('Run all configuration cases', () async {
      final configCases = allCases
          .where((c) => c.category == 'configuration')
          .toList();
      // ignore: avoid_print
      print('\n⚙️ Testing ${configCases.length} configuration cases...');

      for (final testCase in configCases) {
        // Skip error cases
        if (testCase.expected['error_code'] != null) {
          continue;
        }

        // Skip non-Hanafi madhab (same as Python)
        final madhab = testCase.config['madhab'] as String? ?? 'hanafi';
        if (madhab != 'hanafi') {
          // ignore: avoid_print
          print('  ⏭️ ${testCase.id}: Skipped (${madhab} madhab)');
          continue;
        }

        final config = createConfig(testCase.config);
        DartZakatResult? result;

        if (testCase.assetType == 'business') {
          final cashOnHand = testCase.input['cash_on_hand'] as String? ?? '0';
          final inventoryValue =
              testCase.input['inventory_value'] as String? ?? '0';
          final receivables = testCase.input['receivables'] as String? ?? '0';
          final liabilities =
              testCase.input['liabilities_due_now'] as String? ?? '0';
          final hawlSatisfied =
              testCase.input['hawl_satisfied'] as bool? ?? true;

          final business = DartBusiness()
            ..cash(value: Decimal.parse(cashOnHand).toFrb())
            ..inventory(value: Decimal.parse(inventoryValue).toFrb())
            ..receivables(value: Decimal.parse(receivables).toFrb())
            ..debt(value: Decimal.parse(liabilities).toFrb())
            ..hawl(value: hawlSatisfied);

          result = business.calculate(config: config);
        } else if (testCase.assetType == 'gold') {
          final weightGrams = testCase.input['weight_grams'] as String? ?? '0';
          final purity =
              int.tryParse(testCase.input['purity'] as String? ?? '24') ?? 24;
          final liabilities =
              testCase.input['liabilities_due_now'] as String? ?? '0';
          final hawlSatisfied =
              testCase.input['hawl_satisfied'] as bool? ?? true;

          final gold =
              DartPreciousMetals.gold(
                  weightGrams: Decimal.parse(weightGrams).toFrb(),
                )
                ..purity(value: purity)
                ..debt(amount: Decimal.parse(liabilities).toFrb())
                ..hawl(satisfied: hawlSatisfied);

          result = gold.calculate(config: config);
        }

        if (result != null) {
          expect(
            result.isPayable,
            testCase.expected['is_payable'],
            reason:
                '[${testCase.id}] ${testCase.description} - is_payable mismatch',
          );

          // ignore: avoid_print
          print('  ✅ ${testCase.id}: ${testCase.description}');
        }
      }
    });
  });
}
