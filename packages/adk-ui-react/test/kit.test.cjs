// Only an approved kit restyles a host, it sets adk-surface's tokens and never a decision's,
// and a value that is not a colour never reaches CSS.
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');

const { KitProvider, kitStyleVariables, kitA2uiTheme, isApprovedKit, isKitColour } = require('../dist/index.js');

const kit = (overrides = {}) => ({
    id: 'zavora.ai:adk-ui/kit/fintech-pro@0.1.0',
    status: 'approved',
    provenance: { reviewedBy: 'brand-team', approvedAt: '2026-11-16' },
    tokens: {
        colors: {
            primary: '#2F6BFF',
            background: '#ffffff',
            surface: '#ffffff',
            surfaceForeground: '#0f172a',
            muted: '#f1f5f9',
            mutedForeground: '#475569',
            border: '#e2e8f0',
            ring: '#2F6BFF',
            chart: ['#123456', 'url(javascript:alert(1))'],
        },
        typography: { bodyFamily: 'Source Sans 3', monoFamily: 'JetBrains Mono' },
        radius: 'lg',
    },
    ...overrides,
});

describe('brand kits', () => {
    it('map onto adk-surface tokens, never a decision or status colour', () => {
        const variables = kitStyleVariables(kit());
        assert.equal(variables['--accent'], '#2F6BFF');
        assert.equal(variables['--card'], '#ffffff');
        assert.equal(variables['--chart-1'], '#123456');
        assert.equal(variables['--chart-2'], undefined, 'a value that is not a colour is left out');
        assert.equal(variables['--radius'], '12px');
        assert.match(variables['--font'], /^"Source Sans 3"/);
        assert.ok(Object.keys(variables).every((name) => !/decision|pill|waiting/.test(name)));
        assert.deepEqual(kitA2uiTheme(kit()), { primaryColor: '#2F6BFF' });
    });

    it('apply only when approved and signed', () => {
        assert.equal(isApprovedKit(kit()), true);
        assert.equal(isApprovedKit(kit({ status: 'draft' })), false);
        assert.equal(isApprovedKit(kit({ provenance: { reviewedBy: 'brand-team' } })), false);
        assert.throws(() => renderToStaticMarkup(React.createElement(KitProvider, { kit: kit({ status: 'draft' }) }, 'x')), /approved kits only/);
        const markup = renderToStaticMarkup(React.createElement(KitProvider, { kit: kit() }, 'inside'));
        assert.match(markup, /data-adk-ui-kit="zavora.ai:adk-ui\/kit\/fintech-pro@0.1.0"/);
        assert.match(markup, /--accent:#2F6BFF/);
        assert.match(renderToStaticMarkup(React.createElement(KitProvider, { kit: kit({ status: 'draft' }), allowDraft: true }, 'preview')), /preview/);
    });

    it('accept colours only', () => {
        for (const good of ['#fff', '#2F6BFF', 'rgb(0, 0, 0)', 'hsl(210 40% 50%)']) assert.equal(isKitColour(good), true, good);
        for (const bad of ['red; } body {', 'url(x)', 'var(--x)', 'expression(alert(1))', '']) assert.equal(isKitColour(bad), false, bad);
    });
});
