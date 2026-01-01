# Flutter / Dart Usage Guide

The `zakat` library provides a high-performance Dart-Rust bridge, allowing you to use the core Rust logic directly in your Flutter applications.

## Installation

Add the package to your `pubspec.yaml`:

```yaml
dependencies:
  zakat: ^0.1.0 
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

## Notes

*   **Async**: All calls to the Rust bridge are asynchronous `Future`s to prevent blocking the UI thread.
*   **Precision**: Inputs and outputs use `f64` (double). While sufficient for most apps, consider string-based handling if strict financial precision is critical (future update).
*   **Performance**: Calculations run in native Rust, offering near-zero overhead.
