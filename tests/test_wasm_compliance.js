/**
 * Compliance Test Runner for @islamic/zakat WASM Bindings.
 *
 * This test runner loads the golden data from `zakat_suite.json` and verifies
 * that the WASM bindings produce identical results to the Rust core.
 *
 * Usage:
 *   npm test
 */

const fs = require('fs');
const path = require('path');

// Load the WASM module
let zakat;
try {
    zakat = require('./zakat.js');
} catch (e) {
    console.error('Failed to load zakat.js. Run wasm-pack build first.');
    process.exit(1);
}

/**
 * Loads the compliance test suite from JSON.
 */
function loadComplianceSuite() {
    const paths = [
        '../tests/fixtures/zakat_suite.json',
        '../../tests/fixtures/zakat_suite.json',
        path.join(__dirname, '..', 'tests', 'fixtures', 'zakat_suite.json'),
    ];

    for (const p of paths) {
        try {
            const content = fs.readFileSync(p, 'utf8');
            return JSON.parse(content);
        } catch (e) {
            // Continue to next path
        }
    }

    throw new Error(
        'Could not find zakat_suite.json. Run: cargo run -p zakat-test-gen'
    );
}

/**
 * Compares two decimal strings with tolerance.
 */
function assertDecimalEqual(actual, expected, message) {
    const actualNum = parseFloat(actual);
    const expectedNum = parseFloat(expected);
    const diff = Math.abs(actualNum - expectedNum);

    if (diff > 0.0000001) {
        throw new Error(
            `${message}\nExpected: ${expected}\nActual: ${actual}\nDiff: ${diff}`
        );
    }
}

/**
 * Run all compliance tests.
 */
async function runTests() {
    console.log('\n🧪 WASM/TypeScript Compliance Test Runner');
    console.log('━'.repeat(68) + '\n');

    const suite = loadComplianceSuite();
    console.log(`📋 Loaded ${suite.meta.total_cases} test cases`);
    console.log(`   Generated at: ${suite.meta.generated_at}`);
    console.log(`   Schema version: ${suite.meta.schema_version}\n`);

    let passed = 0;
    let failed = 0;
    let skipped = 0;

    // Group by asset type
    const byType = {};
    for (const testCase of suite.cases) {
        if (!byType[testCase.asset_type]) {
            byType[testCase.asset_type] = [];
        }
        byType[testCase.asset_type].push(testCase);
    }

    for (const [assetType, cases] of Object.entries(byType)) {
        console.log(`\n📦 Testing ${cases.length} ${assetType} cases...`);

        for (const testCase of cases) {
            // Skip error cases for now (WASM error handling may differ)
            if (testCase.expected.error_code) {
                skipped++;
                continue;
            }

            try {
                let result;

                // Call appropriate WASM function based on asset type
                if (assetType === 'business') {
                    result = zakat.calculate_single_asset({
                        asset_type: 'business',
                        config: {
                            gold_price_per_gram: testCase.config.gold_price_per_gram,
                            silver_price_per_gram: testCase.config.silver_price_per_gram,
                            madhab: testCase.config.madhab,
                        },
                        input: {
                            cash_on_hand: testCase.input.fields.cash_on_hand || '0',
                            inventory_value: testCase.input.fields.inventory_value || '0',
                            receivables: testCase.input.fields.receivables || '0',
                            liabilities_due_now: testCase.input.liabilities_due_now || '0',
                            hawl_satisfied: testCase.input.hawl_satisfied,
                        },
                    });
                } else if (assetType === 'gold' || assetType === 'silver') {
                    result = zakat.calculate_single_asset({
                        asset_type: assetType,
                        config: {
                            gold_price_per_gram: testCase.config.gold_price_per_gram,
                            silver_price_per_gram: testCase.config.silver_price_per_gram,
                            madhab: testCase.config.madhab,
                        },
                        input: {
                            weight_grams: testCase.input.fields.weight_grams || '0',
                            purity: testCase.input.fields.purity || (assetType === 'gold' ? '24' : '1000'),
                            usage: testCase.input.fields.usage || 'investment',
                            liabilities_due_now: testCase.input.liabilities_due_now || '0',
                            hawl_satisfied: testCase.input.hawl_satisfied,
                        },
                    });
                } else {
                    skipped++;
                    continue;
                }

                // Assert results
                if (result.is_payable !== testCase.expected.is_payable) {
                    throw new Error(
                        `[${testCase.id}] is_payable mismatch: expected ${testCase.expected.is_payable}, got ${result.is_payable}`
                    );
                }

                assertDecimalEqual(
                    result.zakat_due,
                    testCase.expected.zakat_due,
                    `[${testCase.id}] zakat_due mismatch`
                );

                passed++;
            } catch (e) {
                console.error(`  ❌ ${testCase.id}: ${e.message}`);
                failed++;
            }
        }
    }

    // Summary
    console.log('\n' + '━'.repeat(68));
    console.log('📊 Test Summary');
    console.log('━'.repeat(68));
    console.log(`   ✅ Passed:  ${passed}`);
    console.log(`   ❌ Failed:  ${failed}`);
    console.log(`   ⏭️  Skipped: ${skipped}`);
    console.log();

    if (failed === 0) {
        console.log('✅✅✅ ALL WASM COMPLIANCE TESTS PASSED! ✅✅✅\n');
        process.exit(0);
    } else {
        console.log('⚠️  Some tests failed. Review output above.\n');
        process.exit(1);
    }
}

// Run tests
runTests().catch((e) => {
    console.error('Test runner error:', e);
    process.exit(1);
});
