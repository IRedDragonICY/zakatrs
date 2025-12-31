import unittest
from decimal import Decimal, getcontext
import json
import zakatrs

# Set precision just in case, though we pass strings to Rust
getcontext().prec = 28

class TestZakatConfig(unittest.TestCase):
    def test_config_creation(self):
        config = zakatrs.ZakatConfig(gold_price="85.0", silver_price="1.0")
        self.assertEqual(Decimal(config.gold_price), Decimal("85.0"))
        self.assertEqual(Decimal(config.silver_price), Decimal("1.0"))

    def test_config_invalid_price(self):
        with self.assertRaises(ValueError):
            zakatrs.ZakatConfig(gold_price="invalid", silver_price="1.0")

class TestBusinessZakat(unittest.TestCase):
    def setUp(self):
        self.config = zakatrs.ZakatConfig(gold_price="100.0", silver_price="1.0")
        # Gold Nisab = 85 * 100 = 8500
        
    def test_business_payable(self):
        # Cash 5000 + Merch 5000 = 10000. Liabilities 1000. Net 9000.
        # 9000 > 8500 (Nisab). Due = 9000 * 0.025 = 225.
        biz = zakatrs.BusinessZakat(
            cash="5000",
            merchandise="5000",
            receivables="0",
            liabilities="1000"
        )
        result = biz.calculate(self.config)
        
        self.assertTrue(result.is_payable)
        # Rust decimal might format as 9000 or 9000.00
        self.assertEqual(Decimal(result.net_assets), Decimal("9000"))
        self.assertEqual(Decimal(result.zakat_due), Decimal("225.0"))

    def test_business_exempt(self):
        # Net 500 < 595 (Silver Nisab at $1/g)
        biz = zakatrs.BusinessZakat(cash="500")
        result = biz.calculate(self.config)
        self.assertFalse(result.is_payable)
        self.assertEqual(result.zakat_due, "0")

class TestPreciousMetals(unittest.TestCase):
    def setUp(self):
        self.config = zakatrs.ZakatConfig(gold_price="80.0", silver_price="1.0")
        
    def test_gold_zakat(self):
        # 100g Gold.
        gold = zakatrs.PreciousMetals(weight="100", metal_type="gold")
        result = gold.calculate(self.config)
        
        self.assertTrue(result.is_payable)
        self.assertAlmostEqual(float(result.zakat_due), 200.0)

class TestPortfolio(unittest.TestCase):
    def setUp(self):
        self.config = zakatrs.ZakatConfig(gold_price="100.0", silver_price="2.0")
        
    def test_mixed_portfolio(self):
        portfolio = zakatrs.ZakatPortfolio()
        
        # 1. Business: 10,000 Net -> 250 Due
        biz = zakatrs.BusinessZakat(cash="10000")
        portfolio.add(biz)
        
        # 2. Income: 4,000 (Gross). 4000 * 0.025 = 100 Due.
        # Assuming Income Nisab checks total monetary value or individual?
        # The rust lib usually aggregates or checks individual logic.
        # IncomeZakatCalculator checks its own Nisab (85g gold = 8500).
        # 4000 < 8500. So individually it might be exempt if NOT aggregated?
        # But Portfolio should aggregate monetary wealth (Dam' al-Amwal).
        inc = zakatrs.IncomeZakatCalculator(income="4000")
        portfolio.add(inc)
        
        result = portfolio.calculate(self.config)
        
        # Total Assets: 14,000. > 8500.
        # Both should be zakatable due to aggregation.
        # Total Due = 14000 * 0.025 = 350.
        
        self.assertEqual(result.total_assets, "14000")
        self.assertAlmostEqual(float(result.total_zakat_due), 350.0)

    def test_portfolio_json(self):
        portfolio = zakatrs.ZakatPortfolio()
        portfolio.add(zakatrs.BusinessZakat(cash="10000"))
        result = portfolio.calculate(self.config)
        
        data = result.to_dict()
        self.assertIn("total_zakat_due", data)
        self.assertEqual(data["status"], "Complete")

if __name__ == '__main__':
    unittest.main()
