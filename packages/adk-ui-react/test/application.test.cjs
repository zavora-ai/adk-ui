const { describe, it } = require('node:test');
const assert = require('node:assert/strict');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');
const {
    ApplicationRenderer,
    applyUiApplicationUpdate,
    parseUiApplication,
    validateUiApplication,
} = require('../dist/index.js');

function application() {
    return {
        id: 'operations',
        name: 'Operations',
        initial_route: '/',
        navigation: [{ label: 'Overview', route: '/' }],
        pages: [{
            id: 'overview', route: '/', title: 'Overview', template: 'dashboard',
            regions: [{ id: 'main', layout: 'stack', columns: 1, components: [] }],
        }],
        state: { filters: 1 },
    };
}

describe('Application graph', () => {
    it('parses direct and protocol-enveloped applications', () => {
        const app = application();
        assert.equal(parseUiApplication(app).id, 'operations');
        assert.equal(parseUiApplication({ protocol: 'a2ui', application: app }).name, 'Operations');
    });

    it('rejects routes outside the declared page graph', () => {
        const app = application();
        app.navigation.push({ label: 'Missing', route: '/missing' });
        assert.match(validateUiApplication(app)[0], /navigation/);
        assert.equal(parseUiApplication(app), null);
    });

    it('patches state and upserts pages without mutating the source graph', () => {
        const app = application();
        const stateUpdate = applyUiApplicationUpdate(app, { operation: 'patch_state', values: { filters: 2, query: 'open' } });
        const pageUpdate = applyUiApplicationUpdate(stateUpdate, {
            operation: 'upsert_page',
            page: { id: 'detail', route: '/detail', title: 'Detail', regions: [] },
        });
        assert.equal(app.state.filters, 1);
        assert.deepEqual(stateUpdate.state, { filters: 2, query: 'open' });
        assert.equal(pageUpdate.pages.length, 2);
    });

    it('hydrates application component bindings from application state', () => {
        const app = application();
        app.state = { findings: [{ title: 'Disk space is low', severity: 'warning' }] };
        app.pages[0].regions[0].components = [{
            type: 'table',
            columns: [
                { header: 'Finding', accessor_key: 'title', sortable: true },
                { header: 'Severity', accessor_key: 'severity', sortable: true },
            ],
            data: [],
            data_source: { binding_path: '/findings', refresh: { mode: 'push' } },
        }];
        const html = renderToStaticMarkup(React.createElement(ApplicationRenderer, { application: app }));
        assert.match(html, /Disk space is low/);
        assert.match(html, /warning/);
    });
});
