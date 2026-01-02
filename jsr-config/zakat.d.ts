/* tslint:disable */
/* eslint-disable */

export class WasmZakatError {
  private constructor();
  free(): void;
  [Symbol.dispose](): void;
}

/**
 * Calculate Zakat for a portfolio
 * 
 * Adapts the Rust `ZakatPortfolio::calculate_total` to JS.
 * 
 * # Arguments
 * - `config_json`: `ZakatConfig` object
 * - `assets_json`: Array of `PortfolioItem` objects
 */
export function calculate_portfolio_wasm(config_json: any, assets_json: any): any;

/**
 * Helper: Calculate Zakat for a single asset just like the portfolio but simpler
 */
export function calculate_single_asset(config_json: any, asset_json: any): any;

/**
 * Helper: Test if WASM is alive
 */
export function greet(name: string): string;

/**
 * Initialize hooks for better debugging in WASM
 */
export function init_hooks(): void;
