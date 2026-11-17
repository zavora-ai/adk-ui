// The renderers cover exactly the extension components of each adk-ui catalog, as the Rust
// catalogs list them (catalogs/index.json, written and drift-checked by `cargo test`).
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const fs = require('node:fs');

const { adkUiRenderers, insightRenderers, studioRenderers, INSIGHT_CATALOG_ID, STUDIO_CATALOG_ID } = require('../dist/index.js');

const index = JSON.parse(fs.readFileSync(path.join(__dirname, '../../../catalogs/index.json'), 'utf8'));

describe('the renderer registry', () => {
    it('covers every extension component of every catalog, and nothing else', () => {
        assert.deepEqual(index.catalogs.map((catalog) => catalog.id).sort(), Object.keys(adkUiRenderers).sort());
        for (const catalog of index.catalogs) {
            assert.deepEqual(Object.keys(adkUiRenderers[catalog.id]).sort(), catalog.extensions, catalog.label);
        }
    });

    it('keys the insight and studio renderers by their catalog ids', () => {
        assert.deepEqual(Object.keys(insightRenderers), [INSIGHT_CATALOG_ID]);
        assert.deepEqual(Object.keys(studioRenderers), [STUDIO_CATALOG_ID]);
    });
});
