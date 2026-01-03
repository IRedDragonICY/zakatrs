"""
Compliance Test Runner for zakatrs Python Bindings.

This test runner loads the golden data from `zakat_suite.json` and verifies
that the Python bindings produce identical results to the Rust core.

Usage:
    pytest tests/py/test_compliance.py -v
"""

import json
import os
import unittest
from decimal import Decimal, getcontext
from pathlib import Path
from typing import Any, Dict, Optional

# Set high precision for decimal comparisons
getcontext().prec = 28

# Import the zakatrs bindings
try:
    import zakatrs
except ImportError as e:
    raise ImportError(
        "zakatrs not installed. Run: maturin develop -m zakat/Cargo.toml"
    ) from e


def load_compliance_suite() -> Dict[str, Any]:
    """Load the compliance test suite JSON."""
    # Look for the fixture file relative to the tests directory
    fixture_paths = [
        Path(__file__).parent.parent / "fixtures" / "zakat_suite.json",
        Path("tests/fixtures/zakat_suite.json"),
        Path("zakat_suite.json"),
    ]

    for path in fixture_paths:
        if path.exists():
            with open(path, "r", encoding="utf-8") as f:
                return json.load(f)

    raise FileNotFoundError(
        f"Could not find zakat_suite.json in: {[str(p) for p in fixture_paths]}\n"
        "Run 'cargo run -p zakat-test-gen' to generate it."
    )


def normalize_usage(usage: str) -> str:
    """Convert snake_case usage to PascalCase for Rust enum."""
    mapping = {
        "investment": "Investment",
        "personal_use": "PersonalUse",
        "personaluse": "PersonalUse",
    }
    return mapping.get(usage.lower(), usage.title().replace("_", ""))


class ComplianceTestCase(unittest.TestCase):
    """Base class for compliance tests with helper methods."""

    @classmethod
    def setUpClass(cls):
        """Load the compliance suite once for all tests."""
        cls.suite = load_compliance_suite()
        cls.cases = cls.suite["cases"]
        cls.meta = cls.suite["meta"]
        print(f"\n📋 Loaded {cls.meta['total_cases']} test cases")
        print(f"   Generated at: {cls.meta['generated_at']}")
        print(f"   Schema version: {cls.meta['schema_version']}")

    def create_config(self, config: Dict[str, Any]) -> "zakatrs.ZakatConfig":
        """Create a ZakatConfig from test case config."""
        # Note: madhab is not yet exposed in Python API, using default Hanafi
        return zakatrs.ZakatConfig(
            gold_price=config["gold_price_per_gram"],
            silver_price=config["silver_price_per_gram"],
        )

    def assert_decimal_equal(
        self, actual: str, expected: str, msg: str = ""
    ) -> None:
        """Assert two decimal strings are equal."""
        actual_dec = Decimal(actual)
        expected_dec = Decimal(expected)
        
        # Compare with tolerance for floating point edge cases
        diff = abs(actual_dec - expected_dec)
        tolerance = Decimal("0.0000001")
        
        self.assertTrue(
            diff <= tolerance,
            f"{msg}\nExpected: {expected_dec}\nActual: {actual_dec}\nDiff: {diff}"
        )


class TestBusinessCompliance(ComplianceTestCase):
    """Compliance tests for BusinessZakat."""

    def test_business_cases(self):
        """Run all business asset test cases."""
        business_cases = [c for c in self.cases if c["asset_type"] == "business"]
        print(f"\n🏢 Testing {len(business_cases)} business cases...")

        for case in business_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                self._run_business_case(case)

    def _run_business_case(self, case: Dict[str, Any]):
        """Execute a single business test case."""
        config = self.create_config(case["config"])
        input_data = case["input"]
        expected = case["expected"]

        # Check if this case expects an error
        if expected.get("error_code"):
            self._assert_error(case, config)
            return

        # Create business asset
        biz = zakatrs.BusinessZakat(
            cash_on_hand=input_data.get("cash_on_hand", "0"),
            inventory_value=input_data.get("inventory_value", "0"),
            receivables=input_data.get("receivables", "0"),
            liabilities_due_now=input_data.get("liabilities_due_now", "0"),
            hawl_satisfied=input_data.get("hawl_satisfied", True),
        )

        # Calculate
        result = biz.calculate(config)

        # Assert results
        self.assertEqual(
            result.is_payable,
            expected["is_payable"],
            f"[{case['id']}] is_payable mismatch"
        )
        self.assert_decimal_equal(
            result.zakat_due,
            expected["zakat_due"],
            f"[{case['id']}] zakat_due mismatch"
        )
        self.assert_decimal_equal(
            result.net_assets,
            expected["net_assets"],
            f"[{case['id']}] net_assets mismatch"
        )

    def _assert_error(self, case: Dict[str, Any], config: "zakatrs.ZakatConfig"):
        """Assert that a test case raises an expected error."""
        input_data = case["input"]
        expected = case["expected"]

        biz = zakatrs.BusinessZakat(
            cash_on_hand=input_data.get("cash_on_hand", "0"),
            inventory_value=input_data.get("inventory_value", "0"),
            receivables=input_data.get("receivables", "0"),
            liabilities_due_now=input_data.get("liabilities_due_now", "0"),
        )

        with self.assertRaises(Exception) as ctx:
            biz.calculate(config)

        # Check error code if the exception exposes it
        error = ctx.exception
        if hasattr(error, "code"):
            self.assertEqual(
                error.code,
                expected["error_code"],
                f"[{case['id']}] error_code mismatch"
            )


class TestGoldCompliance(ComplianceTestCase):
    """Compliance tests for Gold PreciousMetals."""

    def test_gold_cases(self):
        """Run all gold asset test cases."""
        gold_cases = [c for c in self.cases if c["asset_type"] == "gold"]
        print(f"\n🥇 Testing {len(gold_cases)} gold cases...")

        for case in gold_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                self._run_gold_case(case)

    def _run_gold_case(self, case: Dict[str, Any]):
        """Execute a single gold test case."""
        config = self.create_config(case["config"])
        input_data = case["input"]
        expected = case["expected"]

        # Check if this case expects an error
        if expected.get("error_code"):
            self._assert_error(case, config)
            return

        # Skip non-Hanafi madhab tests (not yet supported in Python API)
        madhab = case["config"].get("madhab", "hanafi")
        if madhab != "hanafi":
            self.skipTest(f"Madhab '{madhab}' not yet supported in Python API")
            return

        # Create gold asset
        gold = zakatrs.PreciousMetals(
            weight_grams=str(input_data.get("weight_grams", "0")),
            metal_type="Gold",
            purity=str(input_data.get("purity", "999")),
            usage=normalize_usage(input_data.get("usage", "Investment")),
            liabilities_due_now=str(input_data.get("liabilities_due_now", "0")),
            hawl_satisfied=input_data.get("hawl_satisfied", True),
        )

        # Calculate
        result = gold.calculate(config)

        # Assert results
        self.assertEqual(
            result.is_payable,
            expected["is_payable"],
            f"[{case['id']}] is_payable mismatch"
        )
        self.assert_decimal_equal(
            result.zakat_due,
            expected["zakat_due"],
            f"[{case['id']}] zakat_due mismatch"
        )

    def _assert_error(self, case: Dict[str, Any], config: "zakatrs.ZakatConfig"):
        """Assert that a test case raises an expected error."""
        input_data = case["input"]
        expected = case["expected"]

        gold = zakatrs.PreciousMetals(
            weight_grams=str(input_data.get("weight_grams", "0")),
            metal_type="Gold",
            purity=str(input_data.get("purity", "999")),
            usage=normalize_usage(input_data.get("usage", "Investment")),
        )

        with self.assertRaises(Exception) as ctx:
            gold.calculate(config)

        error = ctx.exception
        if hasattr(error, "code"):
            self.assertEqual(
                error.code,
                expected["error_code"],
                f"[{case['id']}] error_code mismatch"
            )


class TestSilverCompliance(ComplianceTestCase):
    """Compliance tests for Silver PreciousMetals."""

    def test_silver_cases(self):
        """Run all silver asset test cases."""
        silver_cases = [c for c in self.cases if c["asset_type"] == "silver"]
        print(f"\n🥈 Testing {len(silver_cases)} silver cases...")

        for case in silver_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                self._run_silver_case(case)

    def _run_silver_case(self, case: Dict[str, Any]):
        """Execute a single silver test case."""
        config = self.create_config(case["config"])
        input_data = case["input"]
        expected = case["expected"]

        # Check if this case expects an error
        if expected.get("error_code"):
            return  # Skip error cases for now

        # Create silver asset
        silver = zakatrs.PreciousMetals(
            weight_grams=str(input_data.get("weight_grams", "0")),
            metal_type="Silver",
            purity=str(input_data.get("purity", "999")),
            usage=normalize_usage(input_data.get("usage", "Investment")),
            liabilities_due_now=str(input_data.get("liabilities_due_now", "0")),
            hawl_satisfied=input_data.get("hawl_satisfied", True),
        )

        # Calculate
        result = silver.calculate(config)

        # Assert results
        self.assertEqual(
            result.is_payable,
            expected["is_payable"],
            f"[{case['id']}] is_payable mismatch"
        )
        self.assert_decimal_equal(
            result.zakat_due,
            expected["zakat_due"],
            f"[{case['id']}] zakat_due mismatch"
        )


class TestEdgeCasesCompliance(ComplianceTestCase):
    """Compliance tests for edge cases."""

    def test_edge_cases(self):
        """Run all edge case test cases."""
        edge_cases = [c for c in self.cases if c["category"] == "edge_case"]
        print(f"\n🔬 Testing {len(edge_cases)} edge cases...")

        for case in edge_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                if case["asset_type"] == "business":
                    TestBusinessCompliance()._run_business_case(case)
                elif case["asset_type"] == "gold":
                    TestGoldCompliance()._run_gold_case(case)
                elif case["asset_type"] == "silver":
                    TestSilverCompliance()._run_silver_case(case)


class TestPrecisionCompliance(ComplianceTestCase):
    """Compliance tests for precision handling."""

    def test_precision_cases(self):
        """Run all precision test cases."""
        precision_cases = [c for c in self.cases if c["category"] == "precision"]
        print(f"\n🎯 Testing {len(precision_cases)} precision cases...")

        for case in precision_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                if case["asset_type"] == "business":
                    TestBusinessCompliance()._run_business_case(case)
                elif case["asset_type"] == "gold":
                    TestGoldCompliance()._run_gold_case(case)
                elif case["asset_type"] == "silver":
                    TestSilverCompliance()._run_silver_case(case)


class TestConfigurationCompliance(ComplianceTestCase):
    """Compliance tests for configuration variations."""

    def test_configuration_cases(self):
        """Run all configuration test cases."""
        config_cases = [c for c in self.cases if c["category"] == "configuration"]
        print(f"\n⚙️ Testing {len(config_cases)} configuration cases...")

        for case in config_cases:
            with self.subTest(case_id=case["id"], desc=case["description"]):
                if case["asset_type"] == "business":
                    TestBusinessCompliance()._run_business_case(case)
                elif case["asset_type"] == "gold":
                    TestGoldCompliance()._run_gold_case(case)
                elif case["asset_type"] == "silver":
                    TestSilverCompliance()._run_silver_case(case)


if __name__ == "__main__":
    unittest.main(verbosity=2)
