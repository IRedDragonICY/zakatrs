# Status
status-label = Status
status-payable = PAYABLE
status-exempt = EXEMPT
status-due = Amount Due

# Asset Labels
asset-gold = Gold
asset-silver = Silver
asset-business = Business Assets
asset-livestock = Livestock
asset-agriculture = Agriculture
asset-savings = Savings
asset-investment = Investments
asset-income = Professional Income
asset-mining = Mining Assets
asset-rikaz = Treasure (Rikaz)
asset-other = Other Assets
asset-generic = Asset

# Calculation Steps
step-initial = Initial Value
step-add = Add
step-subtract = Subtract
step-multiply = Multiply
step-result = Result
step-nisab-check = Nisab Threshold Check
step-rate-applied = Rate Applied
step-net-assets = Net Assets
step-total-assets = Total Assets
step-liabilities = Liabilities Due
step-gross-assets = Gross Assets
step-short-term-liabilities = Short-term Liabilities
step-debts-due-now = Debts Due Now
step-net-business-assets = Net Business Assets
step-cash-on-hand = Cash on Hand
step-inventory-value = Inventory Value
step-receivables = Receivables

# Warnings
warn-negative-clamped = Net assets were negative and clamped to zero.

# Livestock
livestock-kind-sheep = Sheep
livestock-kind-cow = Cow
livestock-kind-camel = Camel
camel-age-bint-makhad = Bint Makhad
camel-age-bint-labun = Bint Labun
camel-age-hiqqah = Hiqqah
camel-age-jazaah = Jaza'ah
cow-age-tabi = Tabi'
cow-age-musinnah = Musinnah

# Errors
error-config-missing = Missing configuration field: { $field }.
error-config-gold-positive = Gold price must be strictly positive (> 0).
error-config-silver-positive = Silver price must be strictly positive (> 0).
error-invalid-input = Input must be valid.
error-negative-value = Value must be non-negative.
error-gold-price-required = Gold Price is required for this calculation.
error-silver-price-required = Silver Price is required for this calculation.
error-parse-json = Failed to parse JSON.
error-read-file = Failed to read file.
error-env-var-missing = Environment variable { $name } is missing.
error-env-var-invalid = Environment variable { $name } has invalid format.
error-input-too-long = Input exceeds maximum length of { $max }.
error-invalid-float = Invalid float value.
error-parse-error = Parse error: { $details }.
error-parse-locale = Parse error with { $locale } locale: { $details }.
error-invalid-purity = Purity must be between 1 and 1000.
error-gold-purity = Gold purity must be between 1 and 24.
error-type-required = Type must be specified.
error-type-invalid = Type must be valid.
error-price-required = Price must be set.
error-price-zero = Price for { $animal } must be greater than zero.
error-division-zero = Division by zero.
error-nisab-price-missing = No Nisab price found for date { $date }.
error-date-range-invalid = Start date cannot be after end date.
error-amount-positive = Amount must be positive.
error-fitrah-count = Person count must be greater than 0.
error-fitrah-overflow = Overflow calculating Fitrah total.
error-portfolio-incomplete = Portfolio calculation incomplete. { $failed }/{ $attempted } items failed.
error-portfolio-failed = Portfolio calculation failed completely.
error-asset-not-found = Asset with ID not found.
error-prices-negative = Prices must be non-negative.
