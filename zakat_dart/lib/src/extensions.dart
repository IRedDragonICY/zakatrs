import 'package:decimal/decimal.dart';
import 'package:zakat/src/ffi/api/types.dart';

/// Extensions to bridge Dart's [Decimal] with Rust's [FrbDecimal].
extension FrbDecimalToDecimal on FrbDecimal {
  /// Convert Rust [FrbDecimal] to Dart [Decimal].
  Decimal toDecimal() {
    return Decimal.parse(toString());
  }
}

/// Extensions to bridge Dart's [Decimal] with Rust's [FrbDecimal].
extension DecimalToFrbDecimal on Decimal {
  /// Convert Dart [Decimal] to Rust [FrbDecimal].
  FrbDecimal toFrb() {
    return FrbDecimal.fromString(s: toString());
  }
}

/// Helper for nullable decimals
extension NullableFrbDecimalToDecimal on FrbDecimal? {
  Decimal? toDecimal() {
    if (this == null) return null;
    return this!.toDecimal();
  }
}
