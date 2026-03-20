const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    evaluateChecks,
    resolveDynamicString,
    resolveDynamicValue,
    resolvePath,
} = require('../dist/index.js');

describe('A2ui bindings', () => {
    it('resolves absolute and relative paths', () => {
        const dataModel = { user: { name: 'Ada' } };
        const scope = { name: 'Grace' };

        assert.equal(resolvePath(dataModel, '/user/name', scope), 'Ada');
        assert.equal(resolvePath(dataModel, 'name', scope), 'Grace');
    });

    it('resolves data bindings and formatString', () => {
        const dataModel = { user: { name: 'Ada' }, score: 7 };
        const bound = resolveDynamicValue({ path: '/user/name' }, dataModel);
        assert.equal(bound, 'Ada');

        const formatted = resolveDynamicString(
            { call: 'formatString', args: ['Hello ${/user/name}, score ${add(${/score}, 1)}'] },
            dataModel,
        );
        assert.equal(formatted, 'Hello Ada, score 8');
    });

    it('supports named-argument catalog functions', () => {
        const dataModel = {
            total: 1289.5,
            email: 'ada@example.com',
            createdAt: '2026-03-20T14:30:00Z',
            count: 1,
        };

        assert.equal(
            resolveDynamicValue({ call: 'email', args: { value: { path: '/email' } } }, dataModel),
            true,
        );
        assert.equal(
            resolveDynamicString({ call: 'formatCurrency', args: { value: { path: '/total' }, currency: 'USD' } }, dataModel),
            '$1,289.50',
        );
        assert.equal(
            resolveDynamicString({ call: 'formatDate', args: { value: { path: '/createdAt' }, format: 'MMM dd, yyyy' } }, dataModel),
            'Mar 20, 2026',
        );
        assert.equal(
            resolveDynamicString({
                call: 'pluralize',
                args: {
                    count: { path: '/count' },
                    variants: { one: '1 alert', other: '{count} alerts' },
                },
            }, dataModel),
            '1 alert',
        );
    });

    it('evaluates validation checks and logic expressions', () => {
        const dataModel = {
            form: {
                zip: '02139',
                age: 14,
            },
        };

        assert.deepEqual(
            evaluateChecks([
                {
                    call: 'required',
                    args: { value: { path: '/form/zip' } },
                    message: 'Zip code is required.',
                },
                {
                    call: 'regex',
                    args: { value: { path: '/form/zip' }, pattern: '^[0-9]{5}$' },
                    message: 'Zip code must be five digits.',
                },
                {
                    and: [
                        { call: 'numeric', args: { value: { path: '/form/age' }, min: 13 } },
                        { not: { call: 'numeric', args: { value: { path: '/form/age' }, max: 10 } } },
                    ],
                    message: 'Age must be at least thirteen.',
                },
            ], dataModel),
            { valid: true },
        );

        const failure = evaluateChecks([
            {
                call: 'length',
                args: { value: { path: '/form/zip' }, min: 6 },
                message: 'Zip code is too short.',
            },
        ], dataModel);

        assert.equal(failure.valid, false);
        assert.equal(failure.message, 'Zip code is too short.');
    });
});
