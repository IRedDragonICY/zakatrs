// Package zakat provides Zakat calculation bindings from the zakatrs Rust library.
//
// This package wraps the UniFFI-generated bindings with ergonomic helpers for
// precision-safe decimal handling using shopspring/decimal.
//
// # Usage
//
//	import (
//	    "github.com/IRedDragonICY/zakatrs/zakat_go"
//	    "github.com/shopspring/decimal"
//	)
//
//	func main() {
//	    // Create config
//	    config := zakat.NewConfig("75.50", "0.85")
//
//	    // Calculate business zakat
//	    result, err := zakat.CalculateBusiness(zakat.BusinessInput{
//	        CashOnHand:     "50000",
//	        InventoryValue: "25000",
//	        Receivables:    "10000",
//	        Liabilities:    "5000",
//	        HawlSatisfied:  true,
//	    }, config)
//	    if err != nil {
//	        log.Fatal(err)
//	    }
//
//	    // Result values are strings for precision
//	    fmt.Printf("Zakat Due: %s\n", result.ZakatDue)
//	}
//
// # Decimal Handling
//
// All monetary values are passed as strings at the FFI boundary to preserve
// precision. For calculations in Go, use shopspring/decimal:
//
//	zakatDue, _ := decimal.NewFromString(result.ZakatDue)
package zakat

import (
	"github.com/shopspring/decimal"
)

// ToDecimal converts a string value from the FFI to a shopspring/decimal.Decimal.
// Returns decimal.Zero if parsing fails.
func ToDecimal(s string) decimal.Decimal {
	d, err := decimal.NewFromString(s)
	if err != nil {
		return decimal.Zero
	}
	return d
}

// FromDecimal converts a shopspring/decimal.Decimal to a string for FFI.
func FromDecimal(d decimal.Decimal) string {
	return d.String()
}

// DecimalEqual compares two decimal strings with a tolerance for floating point precision.
// This is useful for test assertions.
func DecimalEqual(actual, expected string, tolerance string) bool {
	actualDec := ToDecimal(actual)
	expectedDec := ToDecimal(expected)
	toleranceDec := ToDecimal(tolerance)
	if toleranceDec.IsZero() {
		toleranceDec = decimal.NewFromFloat(0.0000001)
	}
	diff := actualDec.Sub(expectedDec).Abs()
	return diff.LessThanOrEqual(toleranceDec)
}

// Config holds the Zakat calculation configuration.
// All prices are specified as strings for precision.
type Config struct {
	// GoldPricePerGram is the current gold price per gram
	GoldPricePerGram string
	// SilverPricePerGram is the current silver price per gram
	SilverPricePerGram string
	// Madhab specifies the Islamic school of jurisprudence (hanafi, shafi, maliki, hanbali)
	Madhab string
}

// NewConfig creates a new Config with default Hanafi madhab.
func NewConfig(goldPrice, silverPrice string) Config {
	return Config{
		GoldPricePerGram:   goldPrice,
		SilverPricePerGram: silverPrice,
		Madhab:             "hanafi",
	}
}

// WithMadhab returns a copy of the config with the specified madhab.
func (c Config) WithMadhab(madhab string) Config {
	c.Madhab = madhab
	return c
}

// BusinessInput holds input values for business zakat calculation.
type BusinessInput struct {
	// CashOnHand - liquid cash available
	CashOnHand string
	// InventoryValue - value of business inventory
	InventoryValue string
	// Receivables - money owed to the business
	Receivables string
	// Liabilities - debts due now that should be deducted
	Liabilities string
	// HawlSatisfied - whether one lunar year has passed
	HawlSatisfied bool
}

// GoldInput holds input values for gold zakat calculation.
type GoldInput struct {
	// WeightGrams - weight of gold in grams
	WeightGrams string
	// Purity - karat purity (e.g., "24" for 24k, "18" for 18k)
	Purity string
	// Usage - "Investment" or "PersonalUse"
	Usage string
	// Liabilities - debts due now
	Liabilities string
	// HawlSatisfied - whether one lunar year has passed
	HawlSatisfied bool
}

// SilverInput holds input values for silver zakat calculation.
type SilverInput struct {
	// WeightGrams - weight of silver in grams
	WeightGrams string
	// Purity - millesimal fineness (e.g., "925", "999", "1000")
	Purity string
	// Usage - "Investment" or "PersonalUse"
	Usage string
	// Liabilities - debts due now
	Liabilities string
	// HawlSatisfied - whether one lunar year has passed
	HawlSatisfied bool
}

// ZakatResult holds the result of a zakat calculation.
type ZakatResult struct {
	// IsPayable - whether zakat is due (above nisab and hawl satisfied)
	IsPayable bool
	// ZakatDue - amount of zakat due (string for precision)
	ZakatDue string
	// TotalAssets - total value of assets before liabilities
	TotalAssets string
	// NetAssets - assets after liabilities deduction
	NetAssets string
	// NisabThreshold - the nisab threshold used for comparison
	NisabThreshold string
}

// ZakatDueDecimal returns the ZakatDue as a shopspring/decimal.Decimal.
func (r ZakatResult) ZakatDueDecimal() decimal.Decimal {
	return ToDecimal(r.ZakatDue)
}

// NetAssetsDecimal returns the NetAssets as a shopspring/decimal.Decimal.
func (r ZakatResult) NetAssetsDecimal() decimal.Decimal {
	return ToDecimal(r.NetAssets)
}

// TODO: The following functions will call into the UniFFI-generated bindings.
// They are placeholders until uniffi-bindgen-go generates the actual bindings.
//
// func CalculateBusiness(input BusinessInput, config Config) (ZakatResult, error)
// func CalculateGold(input GoldInput, config Config) (ZakatResult, error)
// func CalculateSilver(input SilverInput, config Config) (ZakatResult, error)
