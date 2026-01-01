import 'package:decimal/decimal.dart';
import 'rust/api/simple.dart';

extension FrbDecimalConversion on FrbDecimal {
  Decimal toDecimal() => Decimal.parse(toString());
}

extension DecimalToFrb on Decimal {
  FrbDecimal toFrb() => FrbDecimal.fromString(s: toString());
}
