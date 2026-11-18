// The renderers cover exactly the extension components of each adk-ui catalog, as the Rust
// catalogs list them (catalogs/index.json, written and drift-checked by `cargo test`).
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const fs = require('node:fs');

const { adkUiRenderers, insightRenderers, spatialRenderers, studioRenderers, INSIGHT_CATALOG_ID, SPATIAL_CATALOG_ID, STUDIO_CATALOG_ID } = require('../dist/index.js');

const index = JSON.parse(fs.readFileSync(path.join(__dirname, '../../../catalogs/index.json'), 'utf8'));

describe('the renderer registry', () => {
    it('covers every extension component the home does not draw natively, and nothing else', () => {
        assert.deepEqual(index.catalogs.map((catalog) => catalog.id).sort(), Object.keys(adkUiRenderers).sort());
        for (const catalog of index.catalogs) {
            const own = catalog.extensions.filter((name) => !catalog.homeDrawn.includes(name));
            assert.deepEqual(Object.keys(adkUiRenderers[catalog.id]).sort(), own, catalog.label);
        }
    });

    it('leaves 3D scenes, cards with depth and 3D layouts to the home', () => {
        const spatial = index.catalogs.find((catalog) => catalog.id === SPATIAL_CATALOG_ID);
        for (const name of ['Scene3D', 'Bar3D', 'Globe', 'Scene', 'DepthCard', 'Stage']) {
            assert.ok(spatial.homeDrawn.includes(name), name);
            assert.equal(adkUiRenderers[SPATIAL_CATALOG_ID][name], undefined, name);
            assert.equal(adkUiRenderers[STUDIO_CATALOG_ID][name], undefined, name);
        }
    });

    it("keys each catalog's renderers by its id", () => {
        assert.deepEqual(Object.keys(insightRenderers), [INSIGHT_CATALOG_ID]);
        assert.deepEqual(Object.keys(studioRenderers), [STUDIO_CATALOG_ID]);
        assert.deepEqual(Object.keys(spatialRenderers), [SPATIAL_CATALOG_ID]);
    });
});
