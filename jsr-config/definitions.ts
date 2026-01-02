/**
 * Represents a decimal number, either as a string or a number.
 * Using string is recommended for precision to avoid floating point errors.
 */
export type Decimal = string | number;

/**
 * Standard for determining the Nisab (minimum threshold) for Zakat eligibility.
 * - `Gold`: 85 grams of gold.
 * - `Silver`: 595 grams of silver.
 * - `LowerOfTwo`: The value of whichever is lower (beneficial for the poor).
 */
export type NisabStandard = "Gold" | "Silver" | "LowerOfTwo";

/**
 * Locale identifier for internationalization.
 * - `en-US`: English (United States)
 * - `id-ID`: Indonesian (Bahasa Indonesia)
 * - `ar-SA`: Arabic (Saudi Arabia)
 */
export type ZakatLocale = "en-US" | "id-ID" | "ar-SA";

/**
 * Configuration for Zakat calculation.
 * Contains prices, standards, and locale settings.
 */
export interface ZakatConfig {
    /** Current market price of Gold per gram. */
    gold_price_per_gram: Decimal;
    /** Current market price of Silver per gram. */
    silver_price_per_gram: Decimal;
    /** Price of Rice per kg (for Zakat Fitrah). */
    rice_price_per_kg?: Decimal | null;
    /** Price of Rice per liter (for Zakat Fitrah). */
    rice_price_per_liter?: Decimal | null;
    /** Which standard to use for Cash/Business Nisab. */
    cash_nisab_standard: NisabStandard;
    /** Override default Gold Nisab (default: 85g). */
    nisab_gold_grams?: Decimal | null;
    /** Override default Silver Nisab (default: 595g). */
    nisab_silver_grams?: Decimal | null;
    /** Override default Agriculture Nisab (default: 653kg). */
    nisab_agriculture_kg?: Decimal | null;
    /** Locale for output formatting. */
    locale?: ZakatLocale | string;
}

/**
 * A loose representation of a Portfolio Item for WASM interoperability.
 * This directly maps to the JSON structure expected by the Rust core.
 */
export interface PortfolioItem {
    /** The type of asset (e.g., 'Business', 'Gold', 'Silver', 'Savings'). */
    type: string;
    /** Dynamic fields depending on the asset type. */
    [key: string]: any;
}
