const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const { formatNumber, formatValue, compact } = require('../dist/index.js');

describe('formatting', () => {
    it('writes numbers and values as the server does', () => {
        assert.equal(formatNumber(4200), '4,200');
        assert.equal(formatNumber(1234567.891), '1,234,567.89');
        assert.equal(formatNumber(-12.5), '-12.5');
        assert.equal(formatValue(42, 'percent'), '42%');
        assert.equal(formatValue(4200, 'currency', 'USD'), 'USD 4,200');
        assert.equal(formatValue(120, undefined, undefined, 'ms'), '120 ms');
        assert.equal(formatValue({ nested: true }), '');
        assert.equal(compact(12500), '12.5k');
    });
});
