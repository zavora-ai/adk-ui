// What each renderer draws from the server's checked properties: text only, no links, images
// or scripts, buttons only as an app's page tabs, and the home's view while a rich renderer
// loads.
const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');

const { adkUiRenderers, INSIGHT_CATALOG_ID, STUDIO_CATALOG_ID } = require('../dist/index.js');

const home = (key) => ({ key: `${key}#1`, component: 'Text', text: 'home view' });
const held = (key, text) => ({ key, component: 'Text', text });
const renderChild = (node) => React.createElement('span', { 'data-node': node.key }, node.text ?? '');

function draw(catalog, component, props, children = []) {
    const renderer = adkUiRenderers[catalog][component];
    const node = { key: 'c', component, children: [home('c'), ...children] };
    return renderToStaticMarkup(React.createElement(() => renderer({ node, props, children, renderChild })));
}

function assertInert(markup, label) {
    assert.doesNotMatch(markup, /<a[\s>]|<img|<script|<iframe|href=/i, label);
    const buttons = markup.match(/<button[^>]*>/g) ?? [];
    assert.ok(buttons.every((button) => /role="tab"/.test(button)), `${label}: only page tabs are buttons`);
}

const samples = {
    Chart: { kind: 'bar', title: 'Revenue', data: [{ month: 'Jan', revenue: 4 }], x: { key: 'month' }, series: [{ key: 'revenue', label: 'Revenue' }] },
    Sparkline: { data: [1, 4, 2], label: 'Latency' },
    Metric: { label: 'Open incidents', value: 4200, format: 'currency', currency: 'USD', delta: { value: -2, goodWhen: 'down' }, status: 'critical' },
    KeyValue: { title: 'Shipment', data: [{ k: 'Carrier', v: 'DHL' }], keyField: 'k', valueField: 'v' },
    Progress: { label: 'Upload', value: 40 },
    Badge: { text: 'Shipped', tone: 'success' },
};

describe('the insight renderers', () => {
    it('draw text only', () => {
        for (const [component, props] of Object.entries(samples)) {
            assertInert(draw(INSIGHT_CATALOG_ID, component, props), component);
        }
    });

    it('write values as the server does', () => {
        const metric = draw(INSIGHT_CATALOG_ID, 'Metric', samples.Metric);
        assert.match(metric, /USD 4,200/);
        assert.match(metric, /-2 · better/);
        assert.match(metric, /Critical/);
        assert.match(draw(INSIGHT_CATALOG_ID, 'KeyValue', samples.KeyValue), /<dt>Carrier<\/dt><dd>DHL<\/dd>/);
        assert.match(draw(INSIGHT_CATALOG_ID, 'Progress', samples.Progress), /aria-valuenow="40"/);
        assert.match(draw(INSIGHT_CATALOG_ID, 'Badge', samples.Badge), />Shipped</);
        assert.match(draw(INSIGHT_CATALOG_ID, 'Sparkline', samples.Sparkline), /aria-label="Latency: 3 values, 1 to 4"/);
    });

    it('title a chart from its checked properties', () => {
        assert.match(draw(INSIGHT_CATALOG_ID, 'Chart', samples.Chart), /<figcaption class="adk-ui-heading">Revenue<\/figcaption>/);
    });
});

describe('the studio renderers', () => {
    it('draw an app as tabs over the pages it holds, drawn by the home', () => {
        const markup = draw(
            STUDIO_CATALOG_ID,
            'App',
            { title: 'Release room', pages: [{ id: 'now', title: 'Now', child: 'p1' }, { id: 'risks', title: 'Risks', child: 'p2' }] },
            [held('p1', 'first page'), held('p2', 'second page')],
        );
        assertInert(markup, 'App');
        assert.equal((markup.match(/role="tab"/g) ?? []).length, 2);
        assert.match(markup, /aria-selected="true"[^>]*>Now</);
        assert.match(markup, /data-node="p1">first page/);
        assert.match(markup, /hidden=""[^>]*><span data-node="p2"/);
    });

    it('draw pages and regions around what they hold', () => {
        const page = draw(STUDIO_CATALOG_ID, 'Page', { title: 'Overview', description: 'This week', children: ['x'] }, [held('x', 'held')]);
        assertInert(page, 'Page');
        assert.match(page, /Overview/);
        assert.match(page, /data-node="x">held/);
        const region = draw(STUDIO_CATALOG_ID, 'Region', { title: 'Health', layout: 'grid', columns: 3, children: ['x'] }, [held('x', 'held')]);
        assert.match(region, /--adk-ui-columns:3/);
    });

    it('list a scene as text, with nothing to press', () => {
        const markup = draw(STUDIO_CATALOG_ID, 'Scene3D', {
            title: 'Bay 4',
            objects: [{ id: 'dock', shape: 'box', label: 'Dock' }, { id: 'crate', shape: 'sphere' }],
        });
        assertInert(markup, 'Scene3D');
        assert.doesNotMatch(markup, /<button/);
        assert.match(markup, /<li>Dock<\/li>/);
    });

    it('include the insight renderers', () => {
        assertInert(draw(STUDIO_CATALOG_ID, 'Metric', samples.Metric), 'studio Metric');
    });
});
