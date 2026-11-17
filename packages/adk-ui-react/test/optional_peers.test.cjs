// `recharts` and `three` are optional peers: the entry point never reaches for them, and a
// chart or scene draws the home's view without them.
const { describe, it, before, after } = require('node:test');
const assert = require('node:assert/strict');
const Module = require('node:module');

const PEERS = ['three', 'recharts'];
const originalResolve = Module._resolveFilename;
const requested = [];

describe('optional peer dependencies', () => {
    let React;
    let renderToStaticMarkup;
    let pkg;

    before(() => {
        Module._resolveFilename = function (request, ...rest) {
            if (PEERS.includes(request.split('/')[0])) {
                requested.push(request);
                const error = new Error(`Cannot find module '${request}'`);
                error.code = 'MODULE_NOT_FOUND';
                throw error;
            }
            return originalResolve.call(this, request, ...rest);
        };
        for (const key of Object.keys(require.cache)) if (key.includes('/dist/')) delete require.cache[key];
        React = require('react');
        renderToStaticMarkup = require('react-dom/server').renderToStaticMarkup;
        pkg = require('../dist/index.js');
    });

    after(() => {
        Module._resolveFilename = originalResolve;
    });

    it('loads the entry point without them', () => {
        assert.ok(pkg.adkUiRenderers);
        assert.deepEqual(requested, []);
    });

    it('draws a chart and a scene as their fallbacks', () => {
        const chart = renderToStaticMarkup(
            React.createElement(pkg.Chart, {
                kind: 'line',
                data: [{ t: 'Jan', v: 3 }],
                x: { key: 't', label: 'Month' },
                series: [{ key: 'v', label: 'Visits' }],
            }),
        );
        assert.match(chart, /<table class="adk-ui-table">/);
        assert.match(chart, /<th scope="col">Visits<\/th>/);
        const scene = renderToStaticMarkup(
            React.createElement(pkg.Scene3D, { objects: [{ id: 'a', shape: 'box', label: 'Dock' }], fallback: 'plain' }),
        );
        assert.match(scene, /plain/);
        assert.match(scene, /<li>Dock<\/li>/);
    });

    it("draws the home's own view of a chart or scene in a DrawnView without them", () => {
        const renderChild = (node) => React.createElement('span', { 'data-node': node.key }, node.text);
        const home = { key: 'c#1', component: 'Text', text: 'home view' };
        const draw = (catalog, component, props) =>
            renderToStaticMarkup(
                React.createElement(() =>
                    pkg.adkUiRenderers[catalog][component]({
                        node: { key: 'c', component, children: [home] },
                        props,
                        children: [],
                        renderChild,
                    }),
                ),
            );
        const chart = draw(pkg.INSIGHT_CATALOG_ID, 'Chart', { kind: 'bar', data: [], x: { key: 'm' }, series: [{ key: 'v' }] });
        assert.match(chart, /data-node="c#1">home view/);
        const scene = draw(pkg.STUDIO_CATALOG_ID, 'Scene3D', { objects: [{ id: 'a', shape: 'box' }] });
        assert.match(scene, /data-node="c#1">home view/);
    });
});
