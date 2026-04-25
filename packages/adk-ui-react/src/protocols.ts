import type { ParsedA2uiMessage } from './a2ui/parser';
import { applyParsedMessages, parseJsonl } from './a2ui/parser';
import type { A2uiComponent, A2uiStore } from './a2ui/store';

type SupportedProtocol = 'a2ui' | 'ag_ui' | 'mcp_apps' | 'adk_ui' | 'awp';

export type ProtocolSurfaceSnapshot = {
    protocol: SupportedProtocol;
    source:
        | 'a2ui_jsonl'
        | 'a2ui_surface'
        | 'ag_ui_custom_surface_event'
        | 'ag_ui_event_payload'
        | 'mcp_apps_structured_content'
        | 'mcp_apps_html_resource'
        | 'adk_ui_surface'
        | 'awp_surface'
        | 'surface_object';
    surfaceId: string;
    catalogId?: string;
    components: A2uiComponent[];
    dataModel?: Record<string, unknown> | null;
    theme?: Record<string, unknown> | null;
    sendDataModel?: boolean;
    bridge?: ProtocolBridgeMetadata | null;
};

export type ProtocolBridgeMetadata = {
    protocolVersion?: string | null;
    hostInfo?: Record<string, unknown> | null;
    hostCapabilities?: Record<string, unknown> | null;
    hostContext?: Record<string, unknown> | null;
    appCapabilities?: Record<string, unknown> | null;
    appInfo?: Record<string, unknown> | null;
    initialized?: boolean;
};

export type ProtocolEnvelope = {
    protocol?: string;
    jsonl?: string;
    events?: Array<Record<string, unknown>>;
    payload?: Record<string, unknown>;
};

const SURFACE_WRAPPER_KEYS = [
    'render_screen_response',
    'render_page_response',
    'response',
    'result',
    'output',
    'payload',
    'data',
];

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function getString(value: unknown): string | undefined {
    return typeof value === 'string' ? value : undefined;
}

function getBoolean(value: unknown): boolean | undefined {
    return typeof value === 'boolean' ? value : undefined;
}

function getRecord(value: unknown): Record<string, unknown> | undefined {
    return isRecord(value) ? value : undefined;
}

function isA2uiComponentArray(value: unknown): value is A2uiComponent[] {
    return (
        Array.isArray(value)
        && value.every(
            (entry) =>
                isRecord(entry)
                && typeof entry.id === 'string'
                && (
                    typeof entry.component === 'string'
                    || isRecord(entry.component)
                ),
        )
    );
}

function normalizeProtocol(value: unknown): SupportedProtocol | null {
    const normalized = getString(value)?.trim().toLowerCase();
    if (!normalized) {
        return null;
    }
    if (normalized === 'a2ui') return 'a2ui';
    if (normalized === 'ag_ui' || normalized === 'ag-ui') return 'ag_ui';
    if (normalized === 'mcp_apps' || normalized === 'mcp-apps') return 'mcp_apps';
    if (normalized === 'adk_ui' || normalized === 'adk-ui') return 'adk_ui';
    if (normalized === 'awp') return 'awp';
    return null;
}

function safeParseJsonl(payload: string): ParsedA2uiMessage[] {
    try {
        return parseJsonl(payload);
    } catch {
        return [];
    }
}

function parseMaybeJson(value: unknown): unknown {
    if (typeof value !== 'string') {
        return value;
    }

    const trimmed = value.trim();
    if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) {
        return value;
    }

    try {
        return JSON.parse(trimmed);
    } catch {
        return value;
    }
}

function surfaceToJsonl(surface: ProtocolSurfaceSnapshot): string {
    const messages: Array<Record<string, unknown>> = [
        {
            createSurface: {
                surfaceId: surface.surfaceId,
                catalogId: surface.catalogId ?? 'catalog',
                theme: surface.theme ?? undefined,
                sendDataModel: surface.sendDataModel ?? true,
            },
        },
    ];

    if (surface.dataModel) {
        messages.push({
            updateDataModel: {
                surfaceId: surface.surfaceId,
                path: '/',
                value: surface.dataModel,
            },
        });
    }

    messages.push({
        updateComponents: {
            surfaceId: surface.surfaceId,
            components: surface.components,
        },
    });

    return `${messages.map((entry) => JSON.stringify(entry)).join('\n')}\n`;
}

function extractSurfaceFromParsedMessages(parsed: ParsedA2uiMessage[]): ProtocolSurfaceSnapshot | null {
    let surfaceId = 'main';
    let catalogId: string | undefined;
    let components: A2uiComponent[] = [];
    let dataModel: Record<string, unknown> | undefined;
    let theme: Record<string, unknown> | undefined;
    let sendDataModel: boolean | undefined;

    for (const entry of parsed) {
        const message = entry.message;
        if ('createSurface' in message) {
            surfaceId = message.createSurface.surfaceId;
            catalogId = message.createSurface.catalogId;
            theme = isRecord(message.createSurface.theme) ? message.createSurface.theme : undefined;
            sendDataModel = getBoolean(message.createSurface.sendDataModel);
        } else if ('updateDataModel' in message) {
            if (!message.updateDataModel.path || message.updateDataModel.path === '/') {
                dataModel = isRecord(message.updateDataModel.value)
                    ? message.updateDataModel.value
                    : undefined;
            }
        } else if ('updateComponents' in message) {
            surfaceId = message.updateComponents.surfaceId;
            components = isA2uiComponentArray(message.updateComponents.components)
                ? message.updateComponents.components
                : [];
        }
    }

    if (components.length === 0) {
        return null;
    }

    return {
        protocol: 'a2ui',
        source: 'a2ui_jsonl',
        surfaceId,
        catalogId,
        components,
        dataModel,
        theme,
        sendDataModel,
    };
}

function extractSurfaceFromA2uiJsonl(payload: string): ProtocolSurfaceSnapshot | null {
    const parsed = safeParseJsonl(payload);
    if (parsed.length === 0) {
        return null;
    }
    return extractSurfaceFromParsedMessages(parsed);
}

function extractSurfaceFromRawSurfaceRecord(
    surface: Record<string, unknown>,
    protocol: SupportedProtocol,
    source: ProtocolSurfaceSnapshot['source'],
    bridge?: ProtocolBridgeMetadata,
): ProtocolSurfaceSnapshot | null {
    const components = isA2uiComponentArray(surface.components) ? surface.components : [];
    if (components.length === 0) {
        return null;
    }

    const surfaceId = getString(surface.surfaceId ?? surface.surface_id) ?? 'main';
    const catalogId = getString(surface.catalogId ?? surface.catalog_id);
    const dataModel = isRecord(surface.dataModel ?? surface.data_model)
        ? (surface.dataModel ?? surface.data_model) as Record<string, unknown>
        : undefined;
    const theme = isRecord(surface.theme) ? surface.theme : undefined;
    const sendDataModel = getBoolean(surface.sendDataModel ?? surface.send_data_model);

    return {
        protocol,
        source,
        surfaceId,
        catalogId,
        components,
        dataModel,
        theme,
        sendDataModel,
        bridge,
    };
}

function extractSurfaceFromAgUiEventValue(
    value: unknown,
    depth = 0,
): ProtocolSurfaceSnapshot | null {
    if (depth > 6) {
        return null;
    }

    const parsed = parseMaybeJson(value);
    if (Array.isArray(parsed)) {
        for (let index = parsed.length - 1; index >= 0; index -= 1) {
            const extracted = extractSurfaceFromAgUiEventValue(parsed[index], depth + 1);
            if (extracted) {
                return extracted;
            }
        }
        return null;
    }

    if (!isRecord(parsed)) {
        return null;
    }

    const wrappedSurface = getRecord(parsed.surface);
    if (wrappedSurface) {
        const extracted = extractSurfaceFromRawSurfaceRecord(
            wrappedSurface,
            'ag_ui',
            'ag_ui_event_payload',
        );
        if (extracted) {
            return extracted;
        }
    }

    const directSurface = extractSurfaceFromRawSurfaceRecord(
        parsed,
        'ag_ui',
        'ag_ui_event_payload',
    );
    if (directSurface) {
        return directSurface;
    }

    const nestedProtocolSurface = extractProtocolSurface(parsed, depth + 1);
    if (
        nestedProtocolSurface
        && !(nestedProtocolSurface.protocol === 'a2ui' && nestedProtocolSurface.source === 'surface_object')
    ) {
        return nestedProtocolSurface;
    }

    for (const nestedKey of SURFACE_WRAPPER_KEYS) {
        if (!(nestedKey in parsed)) {
            continue;
        }
        const extracted = extractSurfaceFromAgUiEventValue(parsed[nestedKey], depth + 1);
        if (extracted) {
            return extracted;
        }
    }

    for (const [key, nestedValue] of Object.entries(parsed)) {
        if (key === 'surface' || SURFACE_WRAPPER_KEYS.includes(key)) {
            continue;
        }

        const extracted = extractSurfaceFromAgUiEventValue(nestedValue, depth + 1);
        if (extracted) {
            return extracted;
        }
    }

    return null;
}

type AgUiToolCallBuffer = {
    toolCallName?: string;
    delta: string;
};

function getAgUiEventType(event: Record<string, unknown>): string {
    return getString(event.type)?.trim().toUpperCase() ?? '';
}

function getAgUiToolCallId(event: Record<string, unknown>): string | undefined {
    return getString(event.toolCallId ?? event.tool_call_id);
}

function getAgUiToolCallName(event: Record<string, unknown>): string | undefined {
    return getString(
        event.toolCallName
        ?? event.tool_call_name
        ?? event.toolName
        ?? event.tool_name
        ?? event.name,
    );
}

function getAgUiToolCallDelta(event: Record<string, unknown>): string | undefined {
    const delta = event.delta ?? event.args ?? event.arguments ?? event.payload ?? event.data;
    if (typeof delta === 'string') {
        return delta;
    }

    if (delta === undefined) {
        return undefined;
    }

    try {
        return JSON.stringify(delta);
    } catch {
        return String(delta);
    }
}

function extractSurfaceFromAgUiEvents(
    events: Array<Record<string, unknown>>,
): ProtocolSurfaceSnapshot | null {
    const toolCallBuffers = new Map<string, AgUiToolCallBuffer>();
    let latestSurface: ProtocolSurfaceSnapshot | null = null;

    for (const event of events) {
        const eventType = getAgUiEventType(event);
        const toolCallId = getAgUiToolCallId(event);
        const toolCallName = getAgUiToolCallName(event);

        if (toolCallId) {
            if (eventType === 'TOOL_CALL_START' && toolCallName) {
                const existing = toolCallBuffers.get(toolCallId) ?? { delta: '' };
                existing.toolCallName = toolCallName;
                toolCallBuffers.set(toolCallId, existing);
            }

            if (eventType === 'TOOL_CALL_ARGS' || eventType === 'TOOL_CALL_CHUNK') {
                const existing = toolCallBuffers.get(toolCallId) ?? { delta: '' };
                if (toolCallName) {
                    existing.toolCallName = toolCallName;
                }
                const delta = getAgUiToolCallDelta(event);
                if (delta) {
                    existing.delta += delta;
                    toolCallBuffers.set(toolCallId, existing);
                    const extracted = extractSurfaceFromAgUiEventValue(existing.delta);
                    if (extracted) {
                        latestSurface = extracted;
                    }
                }
            }

            if (eventType === 'TOOL_CALL_END' || eventType === 'TOOL_CALL_RESULT') {
                const buffered = toolCallBuffers.get(toolCallId);
                if (buffered?.delta) {
                    const extracted = extractSurfaceFromAgUiEventValue(buffered.delta);
                    if (extracted) {
                        latestSurface = extracted;
                    }
                }
                toolCallBuffers.delete(toolCallId);
            }
        }

        if (eventType !== 'CUSTOM') {
            const extracted = extractSurfaceFromAgUiEventValue(event);
            if (extracted) {
                latestSurface = extracted;
            }
            continue;
        }

        if (getString(event.name) !== 'adk.ui.surface') {
            const extracted = extractSurfaceFromAgUiEventValue(event);
            if (extracted) {
                latestSurface = extracted;
            }
            continue;
        }
        const value = event.value;
        if (!isRecord(value)) {
            continue;
        }
        const surface = value.surface;
        if (!isRecord(surface)) {
            continue;
        }

        const extracted = extractSurfaceFromRawSurfaceRecord(
            surface,
            'ag_ui',
            'ag_ui_custom_surface_event',
        );
        if (extracted) {
            latestSurface = extracted;
        }
    }

    return latestSurface;
}

function extractMcpBridgeMetadata(payload: Record<string, unknown>): ProtocolBridgeMetadata | undefined {
    const bridge = getRecord(payload.bridge);
    const protocolVersion = getString(
        bridge?.protocolVersion
        ?? bridge?.protocol_version
        ?? payload.protocolVersion
        ?? payload.protocol_version,
    );
    const hostInfo = getRecord(bridge?.hostInfo ?? bridge?.host_info ?? payload.hostInfo ?? payload.host_info);
    const hostCapabilities = getRecord(
        bridge?.hostCapabilities
        ?? bridge?.host_capabilities
        ?? payload.hostCapabilities
        ?? payload.host_capabilities,
    );
    const hostContext = getRecord(
        bridge?.hostContext
        ?? bridge?.host_context
        ?? payload.hostContext
        ?? payload.host_context,
    );
    const appCapabilities = getRecord(
        bridge?.appCapabilities
        ?? bridge?.app_capabilities
        ?? payload.appCapabilities
        ?? payload.app_capabilities,
    );
    const appInfo = getRecord(
        bridge?.appInfo
        ?? bridge?.app_info
        ?? payload.appInfo
        ?? payload.app_info,
    );
    const initialized = getBoolean(bridge?.initialized ?? payload.initialized);

    if (!protocolVersion && !hostInfo && !hostCapabilities && !hostContext && !appCapabilities && !appInfo && initialized === undefined) {
        return undefined;
    }

    return {
        protocolVersion,
        hostInfo,
        hostCapabilities,
        hostContext,
        appCapabilities,
        appInfo,
        initialized,
    };
}

function extractMcpStructuredContentSurface(
    payload: Record<string, unknown>,
): ProtocolSurfaceSnapshot | null {
    const toolResult = getRecord(payload.toolResult ?? payload.tool_result);
    const bridge = getRecord(payload.bridge);
    const candidates: unknown[] = [
        toolResult?.structuredContent,
        toolResult?.structured_content,
        payload.structuredContent,
        payload.structured_content,
        bridge?.structuredContent,
        bridge?.structured_content,
    ];
    const bridgeMetadata = extractMcpBridgeMetadata(payload);

    for (const candidate of candidates) {
        const parsed = parseMaybeJson(candidate);
        if (!isRecord(parsed)) {
            continue;
        }

        const surface = getRecord(parsed.surface) ?? parsed;
        const extracted = extractSurfaceFromRawSurfaceRecord(
            surface,
            'mcp_apps',
            'mcp_apps_structured_content',
            bridgeMetadata,
        );
        if (extracted) {
            return extracted;
        }
    }

    return null;
}

function extractSurfaceScriptFromHtml(html: string): string | null {
    // Use indexOf-based extraction to avoid polynomial ReDoS with regex on untrusted HTML
    const openTagStart = html.indexOf('<script');
    if (openTagStart === -1) return null;

    const idAttr = html.indexOf('adk-ui-surface', openTagStart);
    if (idAttr === -1) return null;

    const openTagEnd = html.indexOf('>', idAttr);
    if (openTagEnd === -1) return null;

    const closeTag = html.indexOf('</script>', openTagEnd);
    if (closeTag === -1) return null;

    const content = html.substring(openTagEnd + 1, closeTag).trim();
    return content.length > 0 ? content : null;
}

function extractSurfaceFromMcpPayload(payload: Record<string, unknown>): ProtocolSurfaceSnapshot | null {
    const bridgeSurface = extractMcpStructuredContentSurface(payload);
    if (bridgeSurface) {
        return bridgeSurface;
    }

    const resourceReadResponse = isRecord(payload.resourceReadResponse)
        ? payload.resourceReadResponse
        : (isRecord(payload.resource_read_response) ? payload.resource_read_response : null);
    if (!isRecord(resourceReadResponse)) {
        return null;
    }

    const contents = resourceReadResponse.contents;
    if (!Array.isArray(contents) || contents.length === 0) {
        return null;
    }

    const firstContent = contents[0];
    if (!isRecord(firstContent)) {
        return null;
    }

    const html = getString(firstContent.text);
    if (!html) {
        return null;
    }

    const scriptText = extractSurfaceScriptFromHtml(html);
    if (!scriptText) {
        return null;
    }

    let parsed: unknown;
    try {
        parsed = JSON.parse(scriptText);
    } catch {
        return null;
    }

    if (!isRecord(parsed)) {
        return null;
    }

    return extractSurfaceFromRawSurfaceRecord(
        parsed,
        'mcp_apps',
        'mcp_apps_html_resource',
        extractMcpBridgeMetadata(payload),
    );
}

function looksLikeMcpPayload(payload: Record<string, unknown>): boolean {
    return (
        isRecord(payload.resourceReadResponse)
        || isRecord(payload.resource_read_response)
        || isRecord(payload.bridge)
        || isRecord(payload.toolResult)
        || isRecord(payload.tool_result)
        || isRecord(payload.structuredContent)
        || isRecord(payload.structured_content)
    );
}

export function extractProtocolSurface(
    payload: unknown,
    depth = 0,
): ProtocolSurfaceSnapshot | null {
    if (depth > 6) {
        return null;
    }

    if (typeof payload === 'string') {
        const fromJsonl = extractSurfaceFromA2uiJsonl(payload);
        if (fromJsonl) {
            return fromJsonl;
        }

        try {
            return extractProtocolSurface(JSON.parse(payload), depth + 1);
        } catch {
            return null;
        }
    }

    if (!isRecord(payload)) {
        return null;
    }

    const protocol = normalizeProtocol(payload.protocol);

    if (protocol === 'a2ui') {
        if (typeof payload.jsonl === 'string') {
            const fromJsonl = extractSurfaceFromA2uiJsonl(payload.jsonl);
            if (fromJsonl) {
                return fromJsonl;
            }
        }

        const fromSurface = extractSurfaceFromRawSurfaceRecord(payload, 'a2ui', 'a2ui_surface');
        if (fromSurface) {
            return fromSurface;
        }
    }

    if (protocol === 'ag_ui' && Array.isArray(payload.events)) {
        const surface = extractSurfaceFromAgUiEvents(
            payload.events.filter((entry): entry is Record<string, unknown> => isRecord(entry)),
        );
        if (surface) {
            return surface;
        }
    }

    if (protocol === 'mcp_apps') {
        const mcpPayload = isRecord(payload.payload) ? payload.payload : payload;
        const surface = extractSurfaceFromMcpPayload(mcpPayload);
        if (surface) {
            return surface;
        }
    }

    if (protocol === 'adk_ui') {
        const fromSurface = extractSurfaceFromRawSurfaceRecord(payload, 'adk_ui', 'adk_ui_surface');
        if (fromSurface) {
            return fromSurface;
        }
    }

    if (protocol === 'awp') {
        const fromSurface = extractSurfaceFromRawSurfaceRecord(payload, 'awp', 'awp_surface');
        if (fromSurface) {
            return fromSurface;
        }
    }

    if (typeof payload.jsonl === 'string') {
        const fromJsonl = extractSurfaceFromA2uiJsonl(payload.jsonl);
        if (fromJsonl) {
            return fromJsonl;
        }
    }

    const rawProtocol = protocol ?? 'a2ui';
    const fromSurface = extractSurfaceFromRawSurfaceRecord(payload, rawProtocol, 'surface_object');
    if (fromSurface) {
        return fromSurface;
    }

    if (Array.isArray(payload.events)) {
        const surface = extractSurfaceFromAgUiEvents(
            payload.events.filter((entry): entry is Record<string, unknown> => isRecord(entry)),
        );
        if (surface) {
            return surface;
        }
    }

    if (looksLikeMcpPayload(payload)) {
        const surface = extractSurfaceFromMcpPayload(payload);
        if (surface) {
            return surface;
        }
    }

    for (const nestedKey of SURFACE_WRAPPER_KEYS) {
        const nested = payload[nestedKey];
        if (nested === undefined) {
            continue;
        }

        const surface = extractProtocolSurface(parseMaybeJson(nested), depth + 1);
        if (surface) {
            return surface;
        }
    }

    for (const [key, value] of Object.entries(payload)) {
        if (!key.toLowerCase().endsWith('_response')) {
            continue;
        }

        const surface = extractProtocolSurface(parseMaybeJson(value), depth + 1);
        if (surface) {
            return surface;
        }
    }

    return null;
}

export function parseProtocolPayload(payload: unknown): ParsedA2uiMessage[] {
    if (typeof payload === 'string') {
        const fromJsonl = extractSurfaceFromA2uiJsonl(payload);
        if (fromJsonl) {
            return safeParseJsonl(payload);
        }

        try {
            return parseProtocolPayload(JSON.parse(payload));
        } catch {
            return [];
        }
    }

    if (!isRecord(payload)) {
        return [];
    }

    if (typeof payload.jsonl === 'string') {
        return safeParseJsonl(payload.jsonl);
    }

    const surface = extractProtocolSurface(payload as ProtocolEnvelope);
    if (!surface) {
        return [];
    }

    return safeParseJsonl(surfaceToJsonl(surface));
}

export function applySurfaceSnapshot(store: A2uiStore, surface: ProtocolSurfaceSnapshot) {
    store.replaceSurface(surface.surfaceId, surface.components, surface.dataModel ?? {});
}

export function applyProtocolPayload(store: A2uiStore, payload: unknown): ParsedA2uiMessage[] {
    const surface = extractProtocolSurface(payload);
    if (surface) {
        applySurfaceSnapshot(store, surface);
    }

    const parsed = parseProtocolPayload(payload);
    if (parsed.length > 0) {
        applyParsedMessages(store, parsed);
    }
    return parsed;
}
