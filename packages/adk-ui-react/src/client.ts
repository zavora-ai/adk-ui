import type { ParsedA2uiMessage } from './a2ui/parser';
import type { A2uiClientMessagePayload } from './a2ui/events';
import type { A2uiStore } from './a2ui/store';
import { uiEventToMessage, type UiEvent } from './types';
import { UnifiedRenderStore } from './store';

export type UiProtocol = 'adk_ui' | 'a2ui' | 'ag_ui' | 'mcp_apps' | 'awp';

export interface ProtocolClientOptions {
    protocol?: UiProtocol;
    store?: UnifiedRenderStore;
}

export interface OutboundEventOptions {
    surfaceId?: string;
    surfaceVersion?: number;
    threadId?: string;
    runId?: string;
    parentRunId?: string;
    messageId?: string;
    resume?: AgUiResumeEntry[];
}

const DEFAULT_SURFACE_ID = 'main';
export const A2UI_PROTOCOL_VERSION = 'v0.9.1';
export const A2UI_PROTOCOL_VERSION_V1 = 'v1.0';
export const MCP_APPS_PROTOCOL_VERSION = '2026-01-26';
export const MCP_APPS_EXTENSION_ID = 'io.modelcontextprotocol/ui';

export interface A2uiCatalogCapabilities {
    supportedCatalogIds: string[];
    inlineCatalogs?: Array<Record<string, unknown>>;
}

export interface A2uiClientCapabilities {
    'v0.9'?: A2uiCatalogCapabilities;
    'v1.0'?: A2uiCatalogCapabilities;
}

/** @deprecated Pass versioned A2UI capabilities instead. */
export interface LegacyA2uiClientCapabilities {
    supportedCatalogIds: string[];
}

export interface A2uiClientDataModel {
    surfaces: Record<string, Record<string, unknown>>;
}

export interface A2uiTransportMetadata {
    a2uiClientCapabilities?: { 'v0.9': A2uiCatalogCapabilities };
    a2uiRendererCapabilities?: { 'v1.0': A2uiCatalogCapabilities };
    inlineCatalogs?: Array<Record<string, unknown>>;
    a2uiClientDataModel?: A2uiClientDataModel;
    [key: string]: unknown;
}

export interface A2uiClientEnvelope {
    protocol: 'a2ui';
    version: string;
    metadata?: A2uiTransportMetadata;
}

export interface BuildA2uiClientEnvelopeOptions {
    store?: A2uiStore;
    surfaceId?: string;
    metadata?: Record<string, unknown>;
    clientCapabilities?: A2uiClientCapabilities | LegacyA2uiClientCapabilities;
    inlineCatalogs?: Array<Record<string, unknown>>;
    includeDataModel?: boolean;
}

export interface AgUiInterrupt {
    id: string;
    reason: string;
    message?: string;
    toolCallId?: string;
    responseSchema?: Record<string, unknown>;
    expiresAt?: string;
    metadata?: Record<string, unknown>;
}

export interface AgUiResumeEntry {
    interruptId: string;
    status: 'resolved' | 'cancelled';
    payload?: unknown;
}

export type AgUiRunFinishedOutcome =
    | { type: 'success' }
    | { type: 'interrupt'; interrupts: AgUiInterrupt[] };

export interface AgUiAgentCapabilities {
    identity?: {
        name?: string;
        type?: string;
        description?: string;
        version?: string;
        provider?: string;
        documentationUrl?: string;
        metadata?: Record<string, unknown>;
    };
    transport?: {
        streaming?: boolean;
        websocket?: boolean;
        httpBinary?: boolean;
        pushNotifications?: boolean;
        resumable?: boolean;
    };
    tools?: {
        supported?: boolean;
        items?: Array<Record<string, unknown>>;
        parallelCalls?: boolean;
        clientProvided?: boolean;
    };
    output?: {
        structuredOutput?: boolean;
        supportedMimeTypes?: string[];
    };
    state?: {
        snapshots?: boolean;
        deltas?: boolean;
        memory?: boolean;
        persistentState?: boolean;
    };
    multiAgent?: {
        supported?: boolean;
        delegation?: boolean;
        handoffs?: boolean;
        subAgents?: Array<{ name: string; description?: string }>;
    };
    reasoning?: {
        supported?: boolean;
        streaming?: boolean;
        encrypted?: boolean;
    };
    multimodal?: {
        input?: { image?: boolean; audio?: boolean; video?: boolean; pdf?: boolean; file?: boolean };
        output?: { image?: boolean; audio?: boolean };
    };
    execution?: {
        codeExecution?: boolean;
        sandboxed?: boolean;
        maxIterations?: number;
        maxExecutionTime?: number;
    };
    humanInTheLoop?: {
        supported?: boolean;
        approvals?: boolean;
        interventions?: boolean;
        feedback?: boolean;
        interrupts?: boolean;
        approveWithEdits?: boolean;
    };
    custom?: Record<string, unknown>;
}

export interface McpAppsAppInfo {
    name: string;
    version: string;
    title?: string;
    description?: string;
    websiteUrl?: string;
}

export interface McpAppsAppCapabilities {
    availableDisplayModes?: string[];
    tools?: {
        listChanged?: boolean;
    };
    experimental?: Record<string, unknown>;
}

export interface McpAppsInitializeRequest {
    method: 'ui/initialize';
    params: {
        protocolVersion: string;
        appInfo: McpAppsAppInfo;
        appCapabilities: McpAppsAppCapabilities;
    };
}

export interface McpAppsInitializedNotification {
    method: 'ui/notifications/initialized';
    params?: Record<string, never>;
}

export interface McpAppsExtensionCapabilities {
    extensions: {
        [MCP_APPS_EXTENSION_ID]: {
            mimeTypes: string[];
        };
    };
}

export type McpAppsHostNotification =
    | { method: 'ui/notifications/tool-input'; params: { arguments?: Record<string, unknown> } }
    | { method: 'ui/notifications/tool-input-partial'; params: { arguments?: Record<string, unknown> } }
    | { method: 'ui/notifications/tool-result'; params: Record<string, unknown> }
    | { method: 'ui/notifications/tool-cancelled'; params: { reason?: string } }
    | { method: 'ui/notifications/host-context-changed'; params: Record<string, unknown> }
    | { method: 'ui/resource-teardown'; params: Record<string, never> };

export interface BuildMcpAppsInitializeRequestOptions {
    protocolVersion?: string;
    appInfo?: Partial<McpAppsAppInfo>;
    appCapabilities?: McpAppsAppCapabilities;
}

function buildMessageId(surfaceId: string, event: UiEvent, messageId?: string): string {
    return messageId ?? `ui-${surfaceId}-${event.action}-${Date.now()}`;
}

function normalizeA2uiClientCapabilities(
    capabilities: A2uiClientCapabilities | LegacyA2uiClientCapabilities,
    inlineCatalogs?: Array<Record<string, unknown>>,
): A2uiClientCapabilities {
    if ('supportedCatalogIds' in capabilities) {
        return {
            'v0.9': {
                supportedCatalogIds: capabilities.supportedCatalogIds,
                ...(inlineCatalogs?.length ? { inlineCatalogs } : {}),
            },
        };
    }

    if (!inlineCatalogs?.length) return capabilities;

    const version = capabilities['v1.0'] ? 'v1.0' : 'v0.9';
    return {
        ...capabilities,
        [version]: {
            supportedCatalogIds: capabilities[version]?.supportedCatalogIds ?? [],
            ...capabilities[version],
            inlineCatalogs,
        },
    };
}

export function buildMcpAppsExtensionCapabilities(): McpAppsExtensionCapabilities {
    return {
        extensions: {
            [MCP_APPS_EXTENSION_ID]: {
                mimeTypes: ['text/html;profile=mcp-app'],
            },
        },
    };
}

function buildCompatibilityAgUiEvent(
    surfaceId: string,
    event: UiEvent,
    options: OutboundEventOptions,
): Record<string, unknown> {
    return {
        type: 'CUSTOM',
        name: 'adk.ui.event',
        threadId: options.threadId ?? `thread-${surfaceId}`,
        runId: options.runId ?? `run-${surfaceId}`,
        value: {
            surfaceId,
            event,
        },
    };
}

export function buildMcpAppsInitializeRequest(
    options: BuildMcpAppsInitializeRequestOptions = {},
): McpAppsInitializeRequest {
    return {
        method: 'ui/initialize',
        params: {
            protocolVersion: options.protocolVersion ?? MCP_APPS_PROTOCOL_VERSION,
            appInfo: {
                name: options.appInfo?.name ?? 'adk-ui-react',
                version: options.appInfo?.version ?? '2.2.0',
                title: options.appInfo?.title,
                description: options.appInfo?.description,
                websiteUrl: options.appInfo?.websiteUrl,
            },
            appCapabilities: options.appCapabilities ?? {
                availableDisplayModes: ['inline'],
                tools: {
                    listChanged: false,
                },
            },
        },
    };
}

export function buildMcpAppsInitializedNotification(): McpAppsInitializedNotification {
    return {
        method: 'ui/notifications/initialized',
        params: {},
    };
}

export function buildA2uiClientEnvelope(
    message: A2uiClientMessagePayload,
    options: BuildA2uiClientEnvelopeOptions = {},
): A2uiClientEnvelope & A2uiClientMessagePayload {
    const metadata: A2uiTransportMetadata = {
        ...(options.metadata ?? {}),
    };

    if (options.clientCapabilities) {
        const normalizedCapabilities = normalizeA2uiClientCapabilities(
            options.clientCapabilities,
            options.inlineCatalogs,
        );
        if (normalizedCapabilities['v0.9']) {
            metadata.a2uiClientCapabilities = {
                'v0.9': normalizedCapabilities['v0.9'],
            };
        }
        if (normalizedCapabilities['v1.0']) {
            metadata.a2uiRendererCapabilities = {
                'v1.0': normalizedCapabilities['v1.0'],
            };
        }
    }

    if (options.inlineCatalogs && options.inlineCatalogs.length > 0) {
        metadata.inlineCatalogs = options.inlineCatalogs;
    }

    const surfaceId = options.surfaceId
        ?? ('action' in message ? message.action.surfaceId : message.error.surfaceId);

    if (options.store && shouldIncludeA2uiDataModel(options, options.store, surfaceId)) {
        const surface = options.store.getSurface(surfaceId);
        if (surface) {
            metadata.a2uiClientDataModel = {
                surfaces: {
                    [surfaceId]: surface.dataModel,
                },
            };
        }
    }

    return {
        protocol: 'a2ui',
        version: A2UI_PROTOCOL_VERSION,
        ...message,
        ...(Object.keys(metadata).length > 0 ? { metadata } : {}),
    };
}

export function buildOutboundEvent(
    protocol: UiProtocol,
    event: UiEvent,
    options: OutboundEventOptions = {},
): Record<string, unknown> {
    const surfaceId = options.surfaceId ?? DEFAULT_SURFACE_ID;
    const eventEnvelope = {
        surface_ref: { id: surfaceId, version: options.surfaceVersion ?? 1 },
        event,
        occurred_at: new Date().toISOString(),
    };

    switch (protocol) {
        case 'ag_ui':
            return {
                protocol: 'ag_ui',
                input: {
                    threadId: options.threadId ?? `thread-${surfaceId}`,
                    runId: options.runId ?? `run-${surfaceId}`,
                    parentRunId: options.parentRunId,
                    state: {
                        adkUi: {
                            surfaceId,
                            event,
                            eventEnvelope,
                        },
                    },
                    messages: [
                        {
                            id: buildMessageId(surfaceId, event, options.messageId),
                            role: 'user',
                            name: 'adk-ui',
                            content: uiEventToMessage(event),
                        },
                    ],
                    tools: [],
                    context: [],
                    forwardedProps: {
                        source: 'adk-ui-react',
                        surfaceId,
                        uiEvent: event,
                        uiEventEnvelope: eventEnvelope,
                    },
                    ...(options.resume ? { resume: options.resume } : {}),
                },
                event: buildCompatibilityAgUiEvent(surfaceId, event, options),
            };
        case 'mcp_apps':
            if (event.action === 'input_change') {
                return {
                    protocol: 'mcp_apps',
                    method: 'ui/update-model-context',
                    params: {
                        content: [
                            {
                                type: 'text',
                                text: uiEventToMessage(event),
                            },
                        ],
                        structuredContent: {
                            surfaceId,
                            uiEvent: event,
                            uiEventEnvelope: eventEnvelope,
                        },
                    },
                };
            }

            return {
                protocol: 'mcp_apps',
                method: 'ui/message',
                params: {
                    role: 'user',
                    metadata: {
                        surfaceId,
                        uiEvent: event,
                    },
                    content: [
                        {
                            type: 'text',
                            text: uiEventToMessage(event),
                            _meta: {
                                surfaceId,
                                uiEvent: event,
                                uiEventEnvelope: eventEnvelope,
                            },
                        },
                    ],
                },
            };
        case 'a2ui':
        case 'adk_ui':
        default:
            return {
                protocol,
                event: {
                    surfaceId,
                    ...event,
                    envelope: eventEnvelope,
                },
            };
    }
}

export class ProtocolClient {
    private protocol: UiProtocol;
    private readonly store: UnifiedRenderStore;

    constructor(options: ProtocolClientOptions = {}) {
        this.protocol = options.protocol ?? 'adk_ui';
        this.store = options.store ?? new UnifiedRenderStore();
    }

    getProtocol(): UiProtocol {
        return this.protocol;
    }

    setProtocol(protocol: UiProtocol) {
        this.protocol = protocol;
    }

    getStore(): UnifiedRenderStore {
        return this.store;
    }

    applyPayload(payload: unknown): ParsedA2uiMessage[] {
        return this.store.applyPayload(payload);
    }

    buildOutboundEvent(event: UiEvent, options: OutboundEventOptions = {}): Record<string, unknown> {
        return buildOutboundEvent(this.protocol, event, options);
    }
}

export function createProtocolClient(options: ProtocolClientOptions = {}): ProtocolClient {
    return new ProtocolClient(options);
}

function shouldIncludeA2uiDataModel(
    options: BuildA2uiClientEnvelopeOptions,
    store: A2uiStore,
    surfaceId: string,
): boolean {
    if (options.includeDataModel) {
        return true;
    }
    return Boolean(store.getSurface(surfaceId)?.metadata.sendDataModel);
}
