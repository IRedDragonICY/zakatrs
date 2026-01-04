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

# CLI Prompts and Messages
cli-title = ZAKAT CALCULATOR CLI
cli-using-prices = Using prices
cli-gold = Gold
cli-silver = Silver
cli-calculating = Calculating Zakat...
cli-may-allah-accept = May Allah accept your Zakat!
cli-no-assets = No assets added. Exiting.
cli-tip-wizard = Tip: Run with --wizard for a guided step-by-step mode.
cli-loading-portfolio = Loading portfolio from { $path }...
cli-save-snapshot = Save calculation snapshot for audit?
cli-snapshot-saved = Snapshot saved to: { $filename }
cli-offline-mode = Running in offline mode with static prices.
cli-fetching-prices = Fetching live prices...
cli-prices-success = Live prices fetched successfully!
cli-using-fallback = Using fallback prices.

# Menu Options
menu-title = Menu:
menu-add-asset = Add Asset
menu-edit-asset = Edit Asset
menu-save-portfolio = Save Portfolio
menu-load-portfolio = Load Portfolio
menu-calculate-exit = Calculate & Exit
menu-cancel = Cancel

# Asset Types
asset-type-business = Business Assets
asset-type-gold = Gold
asset-type-silver = Silver
asset-type-cash = Cash/Savings
asset-type-investments = Investments
asset-type-agriculture = Agriculture
asset-type-select = Select asset type:

# Form Prompts
form-asset-label = Asset label
form-cash-on-hand = Cash on hand ($):
form-inventory-value = Inventory value ($):
form-receivables = Accounts receivable ($):
form-liabilities = Liabilities/debts due now ($):
form-weight-grams = Weight in grams:
form-total-cash = Total cash/savings ($):
form-market-value = Current market value ($):
form-harvest-weight = Harvest weight in kg:
form-irrigated = Was it irrigated artificially (vs rain-fed)?
form-filename-save = Filename to save (e.g. my_zakat.json):
form-filename-load = Filename to load:

# Status Messages
msg-asset-added = Asset added successfully!
msg-asset-skipped = Skipped asset entry.
msg-asset-updated = Asset updated successfully!
msg-edit-cancelled = Edit cancelled.
msg-portfolio-saved = Portfolio saved to: { $filename }
msg-portfolio-loaded = Portfolio loaded successfully!
msg-no-assets-to-edit = No assets in portfolio to edit.

# Result Labels
result-total-zakat = TOTAL ZAKAT DUE:
result-total-assets = Total Assets:
result-status = Status:
result-payable = PAYABLE
result-exempt = EXEMPT
result-failed = FAILED
result-pay-rate = Pay 2.5%
result-sadaqah = Sadaqah

