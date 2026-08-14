const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    defaultUiKit,
    kitStyleVariables,
    resolveApprovedUiKit,
    resolveKitAssetSource,
} = require('../dist/index.js');

describe('Brand kit runtime', () => {
    it('maps semantic manifest values to scoped renderer variables', () => {
        const variables = kitStyleVariables(defaultUiKit);

        assert.equal(variables['--adk-ui-primary'], '#0f766e');
        assert.equal(variables['--adk-ui-font-heading'], 'Avenir Next, Avenir, sans-serif');
        assert.equal(variables['--adk-ui-chart-1'], '#0f766e');
    });

    it('resolves declared kit assets and rejects malformed kit references', () => {
        const kit = {
            ...defaultUiKit,
            assets: [
                {
                    id: 'logo',
                    kind: 'logo',
                    uri: '/assets/acme.svg',
                    mimeType: 'image/svg+xml',
                    alt: 'Acme',
                },
            ],
        };

        assert.equal(resolveKitAssetSource(kit, 'kit://logo'), '/assets/acme.svg');
        assert.equal(resolveKitAssetSource(kit, 'kit://missing'), null);
        assert.equal(resolveKitAssetSource(kit, 'kit://../logo'), null);
        assert.equal(resolveKitAssetSource(kit, 'https://cdn.example.com/image.png'), 'https://cdn.example.com/image.png');
    });

    it('selects only approved runtime kits', () => {
        const approved = { ...defaultUiKit, id: 'kit://approved' };
        const draft = { ...defaultUiKit, id: 'kit://draft', status: 'draft' };

        assert.equal(resolveApprovedUiKit(approved.id, [approved, draft]).id, approved.id);
        assert.equal(resolveApprovedUiKit(draft.id, [approved, draft]).id, defaultUiKit.id);
        assert.equal(resolveApprovedUiKit('kit://missing', [approved, draft]).id, defaultUiKit.id);
    });
});
