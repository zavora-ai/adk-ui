const { describe, it, before, after } = require('node:test');
const assert = require('node:assert/strict');
const Module = require('node:module');
const fs = require('node:fs');
const path = require('node:path');

const OPTIONAL_PEERS = ['three', 'recharts'];
const originalResolve = Module._resolveFilename;
const requestedPeers = [];

/**
 * `three` and `recharts` are declared as optional peer dependencies, so the
 * package has to stay usable when a host never installs them. These tests hide
 * both modules from the resolver to prove the entry point does not reach for
 * them and that chart/3D components degrade instead of taking down the surface.
 */
function hideOptionalPeers() {
    Module._resolveFilename = function (request, ...rest) {
        const root = request.split('/')[0];
        if (OPTIONAL_PEERS.includes(root)) {
            requestedPeers.push(request);
            const error = new Error(`Cannot find module '${request}'`);
            error.code = 'MODULE_NOT_FOUND';
            throw error;
        }
        return originalResolve.call(this, request, ...rest);
    };
}

describe('Optional peer dependencies', () => {
    let React;
    let renderToStaticMarkup;
    let pkg;

    before(() => {
        hideOptionalPeers();
        React = require('react');
        renderToStaticMarkup = require('react-dom/server').renderToStaticMarkup;
        pkg = require('../dist/index.js');
    });

    after(() => {
        Module._resolveFilename = originalResolve;
    });

    it('loads the package without three or recharts installed', () => {
        assert.equal(typeof pkg.Renderer, 'function');
        assert.equal(typeof pkg.UnifiedRenderStore, 'function');
    });

    it('does not resolve the optional peers at import time', () => {
        assert.deepEqual(requestedPeers, []);
    });

    it('renders a chart component without crashing when recharts is absent', () => {
        const component = {
            type: 'chart',
            id: 'chart-1',
            kind: 'bar',
            data: [{ name: 'a', value: 1 }],
            x_key: 'name',
            y_keys: ['value'],
        };
        const html = renderToStaticMarkup(React.createElement(pkg.Renderer, { component }));
        assert.match(html, /Loading chart/);
    });

    it('renders a 3D scene component without crashing when three is absent', () => {
        const component = {
            type: 'scene_3d',
            id: 'scene-1',
            objects: [],
        };
        const html = renderToStaticMarkup(React.createElement(pkg.Renderer, { component }));
        assert.match(html, /Loading 3D scene/);
    });

    it('ships the degraded fallback so a missing peer explains itself', () => {
        const bundle = fs.readFileSync(path.join(__dirname, '..', 'dist', 'index.js'), 'utf8');
        assert.match(bundle, /adk-ui-optional-renderer/);
        assert.match(bundle, /Install the optional peer dependency/);
        for (const peer of OPTIONAL_PEERS) {
            assert.ok(bundle.includes(`"${peer}"`) || bundle.includes(`'${peer}'`), `expected ${peer} to be named in the fallback path`);
        }
    });
});
