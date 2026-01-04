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

  /// Adds an [FrbDecimal] to this [Decimal].
  /// Example: `Decimal total = myDecimal + result.zakatDue;`
  Decimal operator +(FrbDecimal other) => this + other.toDecimal();

  /// Subtracts an [FrbDecimal] from this [Decimal].
  Decimal operator -(FrbDecimal other) => this - other.toDecimal();

  /// Multiplies this [Decimal] by an [FrbDecimal].
  Decimal operator *(FrbDecimal other) => this * other.toDecimal();

  /// Divides this [Decimal] by an [FrbDecimal].
  /// Note: Division returns Rational, so we convert back to Decimal.
  Decimal operator /(FrbDecimal other) => (this / other.toDecimal()).toDecimal();
}

/// Helper for nullable decimals
extension NullableFrbDecimalToDecimal on FrbDecimal? {
  Decimal? toDecimal() {
    if (this == null) return null;
    return this!.toDecimal();
  }
}
