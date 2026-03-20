const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    A2UI_PROTOCOL_VERSION,
    MCP_APPS_PROTOCOL_VERSION,
    ProtocolClient,
    UnifiedRenderStore,
    buildA2uiClientEnvelope,
    buildValidationFailedEvent,
    buildOutboundEvent,
    buildMcpAppsInitializeRequest,
    buildMcpAppsInitializedNotification,
    createProtocolClient,
} = require('../dist/index.js');

describe('Protocol client', () => {
    it('applies ag_ui payload through unified store', () => {
        const client = createProtocolClient({ protocol: 'ag_ui' });
        const payload = {
            protocol: 'ag_ui',
            events: [
                { type: 'RUN_STARTED', threadId: 'thread-main', runId: 'run-main' },
                {
                    type: 'CUSTOM',
                    name: 'adk.ui.surface',
                    value: {
                        format: 'adk-ui-surface-v1',
                        surface: {
                            surfaceId: 'main',
                            catalogId: 'catalog',
                            components: [{ id: 'root', component: 'Column', children: [] }],
                        },
                    },
                },
                { type: 'RUN_FINISHED', threadId: 'thread-main', runId: 'run-main' },
            ],
        };

        const parsed = client.applyPayload(payload);
        assert.equal(parsed.length, 2);
        const surface = client.getStore().getA2uiStore().getSurface('main');
        assert.ok(surface);
        assert.equal(surface.components.get('root').component, 'Column');
    });

    it('keeps legacy adk_ui payloads for renderer compatibility', () => {
        const store = new UnifiedRenderStore();
        const client = new ProtocolClient({ store });

        const payload = {
            id: 'legacy-1',
            components: [
                {
                    type: 'text',
                    content: 'hello',
                },
            ],
        };

        const parsed = client.applyPayload(payload);
        assert.deepEqual(parsed, []);
        const legacy = client.getStore().getLegacyUiResponse();
        assert.ok(legacy);
        assert.equal(legacy.components[0].type, 'text');
    });

    it('builds outbound events using current protocol mapping', () => {
        const client = createProtocolClient({ protocol: 'mcp_apps' });
        const outbound = client.buildOutboundEvent({
            action: 'button_click',
            action_id: 'confirm',
        });

        assert.equal(outbound.protocol, 'mcp_apps');
        assert.equal(outbound.method, 'ui/message');
        assert.equal(outbound.params.role, 'user');
        assert.equal(outbound.params.metadata.surfaceId, 'main');
        assert.equal(outbound.params.metadata.uiEvent.action, 'button_click');
        assert.equal(outbound.params.content[0].type, 'text');
        assert.equal(outbound.params.content[0]._meta.surfaceId, 'main');
        assert.equal(outbound.params.content[0]._meta.uiEvent.action, 'button_click');
    });

    it('uses model-context updates for non-submitting mcp_apps input changes', () => {
        const outbound = buildOutboundEvent(
            'mcp_apps',
            {
                action: 'input_change',
                name: 'email',
                value: 'a@example.com',
            },
            { surfaceId: 'form-1' },
        );

        assert.equal(outbound.protocol, 'mcp_apps');
        assert.equal(outbound.method, 'ui/update-model-context');
        assert.equal(outbound.params.structuredContent.surfaceId, 'form-1');
        assert.equal(outbound.params.structuredContent.uiEvent.action, 'input_change');
    });

    it('buildOutboundEvent supports explicit ag_ui options', () => {
        const outbound = buildOutboundEvent(
            'ag_ui',
            {
                action: 'tab_change',
                index: 1,
            },
            { surfaceId: 'page-1', threadId: 'thread-x', runId: 'run-x', messageId: 'msg-1' },
        );

        assert.equal(outbound.protocol, 'ag_ui');
        assert.equal(outbound.input.threadId, 'thread-x');
        assert.equal(outbound.input.runId, 'run-x');
        assert.equal(outbound.input.messages[0].id, 'msg-1');
        assert.equal(outbound.input.messages[0].role, 'user');
        assert.equal(outbound.input.state.adkUi.surfaceId, 'page-1');
        assert.equal(outbound.input.state.adkUi.event.action, 'tab_change');
        assert.equal(outbound.input.forwardedProps.uiEvent.index, 1);
        assert.equal(outbound.event.threadId, 'thread-x');
        assert.equal(outbound.event.runId, 'run-x');
        assert.equal(outbound.event.value.surfaceId, 'page-1');
        assert.equal(outbound.event.value.event.action, 'tab_change');
    });

    it('builds MCP Apps initialize primitives for the host bridge', () => {
        const initializeRequest = buildMcpAppsInitializeRequest({
            appInfo: {
                name: 'adk-ui-react-example',
                version: '0.4.0',
                title: 'ADK UI Example View',
            },
        });
        const initialized = buildMcpAppsInitializedNotification();

        assert.equal(initializeRequest.method, 'ui/initialize');
        assert.equal(initializeRequest.params.protocolVersion, MCP_APPS_PROTOCOL_VERSION);
        assert.equal(initializeRequest.params.appInfo.name, 'adk-ui-react-example');
        assert.deepEqual(initializeRequest.params.appCapabilities.availableDisplayModes, ['inline']);
        assert.equal(initialized.method, 'ui/notifications/initialized');
        assert.deepEqual(initialized.params, {});
    });

    it('builds native A2UI envelopes with metadata and data model snapshots', () => {
        const store = createProtocolClient({ protocol: 'a2ui' }).getStore().getA2uiStore();
        store.ensureSurface('main');
        store.configureSurface('main', {
            sendDataModel: true,
            catalogId: 'catalog://standard',
        });
        store.applyUpdateDataModel('main', '/status', 'ready');

        const envelope = buildA2uiClientEnvelope(
            buildValidationFailedEvent('main', '/form/email', 'Email is required.'),
            {
                store,
                clientCapabilities: {
                    supportedCatalogIds: ['catalog://standard'],
                },
                inlineCatalogs: [{ id: 'catalog://custom' }],
            },
        );

        assert.equal(envelope.protocol, 'a2ui');
        assert.equal(envelope.version, A2UI_PROTOCOL_VERSION);
        assert.deepEqual(envelope.metadata.a2uiClientCapabilities, {
            supportedCatalogIds: ['catalog://standard'],
        });
        assert.deepEqual(envelope.metadata.inlineCatalogs, [{ id: 'catalog://custom' }]);
        assert.deepEqual(envelope.metadata.a2uiClientDataModel, {
            surfaces: {
                main: { status: 'ready' },
            },
        });
    });
});
