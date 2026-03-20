const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const { A2uiStore, applyParsedMessages, parseJsonl } = require('../dist/index.js');

describe('A2uiStore', () => {
    it('keeps a stable surface instance', () => {
        const store = new A2uiStore();
        const first = store.ensureSurface('main');
        const second = store.ensureSurface('main');

        assert.strictEqual(first, second);
    });

    it('updates components and data model paths', () => {
        const store = new A2uiStore();
        store.applyUpdateComponents('main', [
            { id: 'root', component: 'Card' },
            { id: 'child', component: 'Text' },
        ]);
        store.applyUpdateDataModel('main', '/status', 'ok');

        const surface = store.getSurface('main');
        assert.equal(surface.components.get('root').component, 'Card');
        assert.deepEqual(surface.dataModel, { status: 'ok' });
    });

    it('tracks createSurface metadata and validation errors', () => {
        const store = new A2uiStore();
        store.ensureSurface('main');
        store.configureSurface('main', {
            catalogId: 'catalog://standard',
            sendDataModel: true,
        });
        store.setValidationError('main', '/form/email', 'Email is required.');
        store.clearValidationError('main', '/form/email');

        const surface = store.getSurface('main');
        assert.equal(surface.metadata.catalogId, 'catalog://standard');
        assert.equal(surface.metadata.sendDataModel, true);
        assert.equal(surface.validationErrors.size, 0);
    });

    it('removes surfaces', () => {
        const store = new A2uiStore();
        store.ensureSurface('main');
        store.removeSurface('main');

        assert.equal(store.getSurface('main'), undefined);
    });
});

describe('A2ui parser', () => {
    it('parses JSONL and applies messages', () => {
        const payload = [
            '{"version":"v0.9","createSurface":{"surfaceId":"main","catalogId":"test","sendDataModel":true}}',
            '{"updateComponents":{"surfaceId":"main","components":[{"id":"root","component":"Card"}]}}',
            '{"updateDataModel":{"surfaceId":"main","path":"/status","value":"ok"}}',
            '{"deleteSurface":{"surfaceId":"main"}}',
        ].join('\n');

        const parsed = parseJsonl(payload);
        assert.equal(parsed.length, 4);

        const store = new A2uiStore();
        applyParsedMessages(store, parsed.slice(0, 3));
        const surface = store.getSurface('main');
        assert.equal(surface.components.get('root').component, 'Card');
        assert.deepEqual(surface.dataModel, { status: 'ok' });
        assert.equal(surface.metadata.catalogId, 'test');
        assert.equal(surface.metadata.sendDataModel, true);

        applyParsedMessages(store, parsed.slice(3));
        assert.equal(store.getSurface('main'), undefined);
    });
});
