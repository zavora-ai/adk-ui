import { A2uiStore } from './a2ui/store';
import type { ParsedA2uiMessage } from './a2ui/parser';
import type { ProtocolSurfaceSnapshot } from './protocols';
import { applySurfaceSnapshot, extractProtocolSurface, parseProtocolPayload } from './protocols';
import type { UiResponse } from './types';

function isRecord(value: unknown): value is Record<string, unknown> {
    return typeof value === 'object' && value !== null;
}

function isLegacyComponentArray(value: unknown): value is UiResponse['components'] {
    return Array.isArray(value) && value.every((entry) => isRecord(entry) && typeof entry.type === 'string');
}

function getUiTheme(value: unknown): UiResponse['theme'] | undefined {
    if (value === 'light' || value === 'dark' || value === 'system') {
        return value;
    }
    return undefined;
}

function extractLegacyUiResponse(payload: unknown): UiResponse | null {
    if (!isRecord(payload)) {
        return null;
    }

    if (isLegacyComponentArray(payload.components)) {
        return {
            id: typeof payload.id === 'string' ? payload.id : undefined,
            theme: getUiTheme(payload.theme),
            components: payload.components,
        };
    }

    if (
        isRecord(payload.payload)
        && isLegacyComponentArray(payload.payload.components)
    ) {
        return {
            id: typeof payload.payload.id === 'string' ? payload.payload.id : undefined,
            theme: getUiTheme(payload.payload.theme),
            components: payload.payload.components,
        };
    }

    return null;
}

export class UnifiedRenderStore {
    private readonly a2uiStore: A2uiStore;
    private protocolSurface: ProtocolSurfaceSnapshot | null = null;
    private legacyUiResponse: UiResponse | null = null;

    constructor(a2uiStore: A2uiStore = new A2uiStore()) {
        this.a2uiStore = a2uiStore;
    }

    getA2uiStore(): A2uiStore {
        return this.a2uiStore;
    }

    getLegacyUiResponse(): UiResponse | null {
        return this.legacyUiResponse;
    }

    getProtocolSurface(): ProtocolSurfaceSnapshot | null {
        return this.protocolSurface;
    }

    clearLegacyUiResponse() {
        this.legacyUiResponse = null;
    }

    clearProtocolSurface() {
        this.protocolSurface = null;
    }

    applyPayload(payload: unknown): ParsedA2uiMessage[] {
        const surface = extractProtocolSurface(payload);
        if (surface) {
            this.protocolSurface = surface;
            this.legacyUiResponse = null;
            applySurfaceSnapshot(this.a2uiStore, surface);
            return parseProtocolPayload(payload);
        }

        const legacy = extractLegacyUiResponse(payload);
        if (legacy) {
            this.protocolSurface = null;
            this.legacyUiResponse = legacy;
        }
        return [];
    }
}
