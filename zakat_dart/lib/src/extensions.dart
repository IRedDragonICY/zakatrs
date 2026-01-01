import 'package:decimal/decimal.dart';
import 'rust/api/zakat.dart';

extension FrbDecimalConversion on FrbDecimal {
  Decimal toDecimal() => Decimal.parse(toString());
}

extension DecimalToFrb on Decimal {
  FrbDecimal toFrb() => FrbDecimal.fromString(s: toString());
}
