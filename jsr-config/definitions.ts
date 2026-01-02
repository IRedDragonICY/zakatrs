export type Decimal = string | number;

export type NisabStandard = "Gold" | "Silver" | "LowerOfTwo";

export type ZakatLocale = "en-US" | "id-ID" | "ar-SA";

export interface ZakatConfig {
    gold_price_per_gram: Decimal;
    silver_price_per_gram: Decimal;
    rice_price_per_kg?: Decimal | null;
    rice_price_per_liter?: Decimal | null;
    cash_nisab_standard: NisabStandard;
    nisab_gold_grams?: Decimal | null;
    nisab_silver_grams?: Decimal | null;
    nisab_agriculture_kg?: Decimal | null;
    locale?: ZakatLocale | string;
}

export interface PortfolioItem {
    type: string;
    [key: string]: any;
}
