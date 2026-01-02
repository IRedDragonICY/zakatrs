/**
 * # Zakat
 * 
 * A Fiqh-compliant, type-safe Zakat calculation library for the web.
 * Powered by Rust and WebAssembly.
 * 
 * ## Features
 * - **Fiqh Compliant**: Adheres to scholarly standards for Gold, Silver, Business, and more.
 * - **Type Safe**: Explicit TypeScript definitions for all configurations and inputs.
 * - **Fast**: Core logic runs in WebAssembly.
 * 
 * ## Usage
 * 
 * ```typescript
 * import { calculate_single_asset, type ZakatConfig } from "@islam/zakat";
 * 
 * const config: ZakatConfig = {
 *   gold_price_per_gram: 100,
 *   silver_price_per_gram: 1.5,
 *   cash_nisab_standard: "Silver"
 * };
 * 
 * // Calculate Zakat on Cash
 * const result = calculate_single_asset(config, "Savings", {
 *   amount: 5000,
 *   interest: 0,
 *   hawl_satisfied: true
 * });
 * 
 * console.log(result);
 * ```
 * 
 * @module
 */
// @deno-types="./zakat.d.ts"
export {
    calculate_portfolio_wasm,
    calculate_single_asset,
    greet
} from "./zakat.js";

export type { ZakatConfig, PortfolioItem, NisabStandard, ZakatLocale } from "./definitions.ts";
