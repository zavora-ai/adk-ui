const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    UnifiedRenderStore,
    extractProtocolSurface,
    parseProtocolPayload,
} = require('../dist/index.js');

describe('Protocol payload adapter', () => {
    it('extracts ag_ui custom surface events as native protocol surfaces', () => {
        const payload = {
            protocol: 'ag_ui',
            events: [
                { type: 'RUN_STARTED', threadId: 'thread-1', runId: 'run-1' },
                {
                    type: 'CUSTOM',
                    name: 'adk.ui.surface',
                    value: {
                        format: 'adk-ui-surface-v1',
                        surface: {
                            surfaceId: 'main',
                            catalogId: 'catalog',
                            components: [{ id: 'root', component: 'Column', children: [] }],
                            dataModel: { status: 'ok' },
                        },
                    },
                },
                { type: 'RUN_FINISHED', threadId: 'thread-1', runId: 'run-1' },
            ],
        };

        const surface = extractProtocolSurface(payload);
        assert.ok(surface);
        assert.equal(surface.protocol, 'ag_ui');
        assert.equal(surface.source, 'ag_ui_custom_surface_event');
        assert.equal(surface.surfaceId, 'main');
        assert.deepEqual(surface.dataModel, { status: 'ok' });

        const parsed = parseProtocolPayload(payload);
        assert.equal(parsed.length, 3);
    });

    it('extracts mcp_apps html resource payload and applies native surface state to the store', () => {
        const surface = {
            surfaceId: 'main',
            catalogId: 'catalog',
            components: [{ id: 'root', component: 'Column', children: [] }],
            dataModel: { phase: 'ready' },
        };
        const html = `<!doctype html><html><body><script id="adk-ui-surface" type="application/json">${JSON.stringify(surface)}</script></body></html>`;
        const payload = {
            protocol: 'mcp_apps',
            payload: {
                resourceReadResponse: {
                    contents: [{ text: html }],
                },
            },
        };

        const store = new UnifiedRenderStore();
        const parsed = store.applyPayload(payload);
        assert.equal(parsed.length, 3);

        const protocolSurface = store.getProtocolSurface();
        assert.ok(protocolSurface);
        assert.equal(protocolSurface.protocol, 'mcp_apps');
        assert.equal(protocolSurface.source, 'mcp_apps_html_resource');

        const state = store.getA2uiStore().getSurface('main');
        assert.ok(state);
        assert.equal(state.components.get('root').component, 'Column');
        assert.deepEqual(state.dataModel, { phase: 'ready' });
    });

    it('extracts mcp_apps structured content before falling back to html resources', () => {
        const surface = {
            surfaceId: 'bridge-main',
            catalogId: 'catalog',
            components: [{ id: 'root', component: 'Column', children: [] }],
            dataModel: { phase: 'bridge' },
        };
        const payload = {
            protocol: 'mcp_apps',
            payload: {
                bridge: {
                    protocolVersion: '2025-11-21',
                    appInfo: {
                        name: 'adk-ui-react-example',
                        version: '0.4.0',
                    },
                    structuredContent: {
                        surface,
                    },
                    hostContext: {
                        theme: 'light',
                        displayMode: 'inline',
                    },
                    initialized: true,
                },
                resourceReadResponse: {
                    contents: [{ text: '<html><body>fallback only</body></html>' }],
                },
            },
        };

        const extracted = extractProtocolSurface(payload);
        assert.ok(extracted);
        assert.equal(extracted.protocol, 'mcp_apps');
        assert.equal(extracted.source, 'mcp_apps_structured_content');
        assert.equal(extracted.surfaceId, 'bridge-main');
        assert.equal(extracted.bridge.protocolVersion, '2025-11-21');
        assert.equal(extracted.bridge.hostContext.theme, 'light');
        assert.equal(extracted.bridge.appInfo.name, 'adk-ui-react-example');
        assert.equal(extracted.bridge.initialized, true);

        const store = new UnifiedRenderStore();
        const parsed = store.applyPayload(payload);
        assert.equal(parsed.length, 3);
        assert.equal(store.getA2uiStore().getSurface('bridge-main').dataModel.phase, 'bridge');
    });

    it('extracts direct ag_ui render payloads that use nested component objects', () => {
        const payload = {
            protocol: 'ag_ui',
            surfaceId: 'form',
            components: [
                {
                    id: 'main-card',
                    component: {
                        Card: {
                            title: 'Support Intake Flow',
                            description: 'Severity & Timeline Intake Form',
                        },
                    },
                },
                {
                    id: 'root',
                    component: {
                        Column: {
                            children: ['main-card'],
                        },
                    },
                },
            ],
        };

        const surface = extractProtocolSurface(payload);
        assert.ok(surface);
        assert.equal(surface.protocol, 'ag_ui');
        assert.equal(surface.surfaceId, 'form');
        assert.equal(surface.components[0].id, 'main-card');
        assert.equal(surface.components[1].component.Column.children[0], 'main-card');
    });

    it('extracts ag_ui stable tool call result events as protocol surfaces', () => {
        const payload = {
            protocol: 'ag_ui',
            events: [
                {
                    type: 'TOOL_CALL_START',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                    toolCallName: 'render_screen',
                },
                {
                    type: 'TOOL_CALL_RESULT',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                    messageId: 'msg-tool-1',
                    role: 'tool',
                    content: JSON.stringify({
                        surface: {
                            surfaceId: 'tool-result',
                            catalogId: 'catalog',
                            components: [{ id: 'root', component: 'Column', children: [] }],
                            dataModel: { status: 'live' },
                        },
                    }),
                },
            ],
        };

        const surface = extractProtocolSurface(payload);
        assert.ok(surface);
        assert.equal(surface.protocol, 'ag_ui');
        assert.equal(surface.source, 'ag_ui_event_payload');
        assert.equal(surface.surfaceId, 'tool-result');
        assert.deepEqual(surface.dataModel, { status: 'live' });

        const store = new UnifiedRenderStore();
        const parsed = store.applyPayload(payload);
        assert.equal(parsed.length, 3);
        assert.equal(store.getA2uiStore().getSurface('tool-result').dataModel.status, 'live');
    });

    it('reconstructs ag_ui tool call chunk sequences into protocol surfaces', () => {
        const payload = {
            protocol: 'ag_ui',
            events: [
                {
                    type: 'TOOL_CALL_START',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                    toolCallName: 'render_screen',
                },
                {
                    type: 'TOOL_CALL_CHUNK',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                    toolCallName: 'render_screen',
                    delta: '{"surface":{"surfaceId":"chunk-main","catalogId":"catalog","components":[',
                },
                {
                    type: 'TOOL_CALL_CHUNK',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                    delta: '{"id":"root","component":"Column","children":[]}],"dataModel":{"mode":"chunked"}}}',
                },
                {
                    type: 'TOOL_CALL_END',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    toolCallId: 'tool-1',
                },
            ],
        };

        const surface = extractProtocolSurface(payload);
        assert.ok(surface);
        assert.equal(surface.protocol, 'ag_ui');
        assert.equal(surface.source, 'ag_ui_event_payload');
        assert.equal(surface.surfaceId, 'chunk-main');
        assert.deepEqual(surface.dataModel, { mode: 'chunked' });

        const store = new UnifiedRenderStore();
        const parsed = store.applyPayload(payload);
        assert.equal(parsed.length, 3);
        assert.equal(store.getA2uiStore().getSurface('chunk-main').dataModel.mode, 'chunked');
    });

    it('extracts ag_ui message snapshots that carry nested function response surfaces', () => {
        const payload = {
            protocol: 'ag_ui',
            events: [
                {
                    type: 'MESSAGES_SNAPSHOT',
                    threadId: 'thread-1',
                    runId: 'run-1',
                    messages: [
                        {
                            role: 'assistant',
                            content: [
                                {
                                    functionResponse: {
                                        name: 'render_screen',
                                        response: {
                                            surface: {
                                                surfaceId: 'snapshot-main',
                                                catalogId: 'catalog',
                                                components: [{ id: 'root', component: 'Column', children: [] }],
                                                dataModel: { mode: 'snapshot' },
                                            },
                                        },
                                    },
                                },
                            ],
                        },
                    ],
                },
            ],
        };

        const surface = extractProtocolSurface(payload);
        assert.ok(surface);
        assert.equal(surface.protocol, 'ag_ui');
        assert.equal(surface.source, 'ag_ui_event_payload');
        assert.equal(surface.surfaceId, 'snapshot-main');
        assert.deepEqual(surface.dataModel, { mode: 'snapshot' });
    });

    it('returns empty output for unsupported payloads', () => {
        const parsed = parseProtocolPayload({ protocol: 'unsupported' });
        assert.deepEqual(parsed, []);
    });
});
