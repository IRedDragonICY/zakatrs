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
 *   goldPricePerGram: "100",
 *   silverPricePerGram: "1.5",
 *   cashNisabStandard: "silver"
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

// Export TypeScript types generated from Rust via typeshare
export * from "./types.ts";
