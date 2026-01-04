# Flutter / Dart Usage Guide

The `zakat` library provides a high-performance Dart-Rust bridge, allowing you to use the core Rust logic directly in your Flutter applications.

## Installation

Add the package to your `pubspec.yaml`:

```yaml
dependencies:
  zakat: ^1.2.0
```
*Note: Ensure the package version matches the latest release on pub.dev.*

## Initialization

Before calling any Zakat functions, you must initialize the Rust bridge. This is best done in your `main()` function.

```dart
import 'package:flutter/material.dart';
import 'package:zakat/main.dart'; // Or wherever your init wrapper is
import 'package:zakat/src/rust/frb_generated.dart'; // Import generated bridge

void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Initialize the Rust bridge
  await RustLib.init();
  
  runApp(const MyApp());
}
```

## Basic Usage

The API exposes high-level functions that take Dart native types (`double`) and return structured results via `DartZakatResult`.

### 1. Calculate Business Zakat

```dart
import 'package:zakat/src/rust/api/simple.dart';

Future<void> calculateBusiness() async {
  final result = await calculateBusinessZakat(
    cash: 10000.0,
    inventory: 5000.0,
    receivables: 2000.0,
    liabilities: 1000.0,
    goldPrice: 85.0,   // USD/gram (Example)
    silverPrice: 1.0,  // USD/gram
  );

  print('Is Payable: ${result.isPayable}');
  print('Zakat Due: ${result.zakatDue}');
  print('Nisab Threshold: ${result.nisabThreshold}');
}
```

### 2. Calculate Savings Zakat

Calculate Zakat on cash savings or bank balances.

```dart
Future<void> calculateSavings() async {
  final result = await calculateSavingsZakat(
    cashInHand: 5000.0,
    bankBalance: 12000.0,
    goldPrice: 85.0,
    silverPrice: 1.0,
  );

  if (result.isPayable) {
    print('You must pay: ${result.zakatDue}');
  } else {
    print('Total wealth ${result.wealthAmount} is below Nisab ${result.nisabThreshold}');
  }
}
```

### 3. Check Nisab Thresholds

Get the current Nisab values based on live gold/silver prices.

```dart
Future<void> checkNisab() async {
  final (goldNisab, silverNisab) = await getNisabThresholds(85.0, 1.0);
  
  print('Gold Nisab: $goldNisab');
  print('Silver Nisab: $silverNisab');
}
```

## Type Mapping (Rust → Dart)

Understanding how Rust types are serialized to Dart:

| Rust Type | Dart Type | Notes |
|:----------|:----------|:------|
| `Decimal` | `String` | Preserved as string for precision. Parse with `double.parse()` or a `Decimal` package. |
| `Option<T>` | `T?` | Nullable types. |
| `Uuid` | `String` | Standard UUID format. |
| `Vec<T>` | `List<T>` | Lists serialize directly. |
| `bool` | `bool` | Direct mapping. |
| `HashMap<K,V>` | `Map<K,V>` | Maps serialize directly. |

> **Why `String` for Decimal?**
> Dart's `double` type (IEEE 754 float64) loses precision after ~15 significant digits. For financial calculations requiring 28-digit precision, values are passed as strings.

## Precision Handling

For high-precision arithmetic, use the `decimal` package:

```dart
import 'package:decimal/decimal.dart';

Future<void> preciseMath() async {
  final result = await calculateBusinessZakat(...);
  
  // Parse string-based Decimal values
  final zakatDue = Decimal.parse(result.zakatDueString);
  final totalAssets = Decimal.parse(result.totalAssetsString);
  
  // Precise percentage calculation
  final percentage = (zakatDue / totalAssets * Decimal.fromInt(100))
      .toDecimal(scaleOnInfinitePrecision: 4);
  print('Zakat is $percentage% of assets');
}
```

## Data Types

### `DartZakatResult`

Verified structure returned by calculation functions:

| Field | Type | Description |
| :--- | :--- | :--- |
| `zakatDue` | `double` | Total Zakat amount to pay. |
| `isPayable` | `bool` | Whether the assets exceed the Nisab. |
| `nisabThreshold` | `double` | The threshold value used for this calculation. |
| `wealthAmount` | `double` | Total net assets calculated. |
| `limitName` | `String` | Debug name of the limit used (e.g., "Nisab (Silver)"). |

## Error Handling

Errors from the Rust library are propagated as exceptions:

```dart
import 'package:zakat/src/rust/api/simple.dart';

Future<void> handleErrors() async {
  try {
    final result = await calculateBusinessZakat(
      cash: -1000.0, // Invalid: negative value
      inventory: 0.0,
      receivables: 0.0,
      liabilities: 0.0,
      goldPrice: 85.0,
      silverPrice: 1.0,
    );
  } on FfiException catch (e) {
    // Structured error from Rust FfiZakatError
    // e.message contains the error description
    print('Error: ${e.message}');
    
    // Extract structured data from the error message
    // (The error includes code, field, and hint information)
    showSnackBar(context, 'Calculation failed: ${e.message}');
  } catch (e) {
    // Generic error fallback
    print('Unexpected error: $e');
  }
}
```

## Debugging

### Enable Detailed Logging

To trace calculation issues during development:

```dart
import 'package:flutter/foundation.dart';

Future<void> debugCalculation() async {
  if (kDebugMode) {
    print('Starting Zakat calculation...');
    print('Gold Price: 85.0/g, Silver Price: 1.0/g');
  }
  
  final result = await calculateBusinessZakat(
    cash: 10000.0,
    inventory: 5000.0,
    receivables: 2000.0,
    liabilities: 1000.0,
    goldPrice: 85.0,
    silverPrice: 1.0,
  );
  
  if (kDebugMode) {
    print('Calculation result:');
    print('  Is Payable: ${result.isPayable}');
    print('  Net Wealth: ${result.wealthAmount}');
    print('  Nisab: ${result.nisabThreshold}');
    print('  Zakat Due: ${result.zakatDue}');
  }
}
```

### Rust FFI Trace

For low-level debugging, enable Rust's tracing in the native extension (requires recompilation with `RUST_LOG=debug`).

## Notes

*   **Async**: All calls to the Rust bridge are asynchronous `Future`s to prevent blocking the UI thread.
*   **Precision**: For simple use cases, `double` is sufficient. For strict financial calculations, use the `decimal` package with string-based inputs.
*   **Performance**: Calculations run in native Rust, offering near-zero overhead.
*   **Thread Safety**: The Rust bridge is thread-safe; you can call it from isolates.

