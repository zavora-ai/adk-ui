import type { A2uiActionResponsePayload } from './events';
import type { CallFunctionMessage } from './parser';

export type A2uiComponent = Record<string, unknown> & {
    id: string;
    component: string | Record<string, unknown>;
};

export interface SurfaceMetadata {
    catalogId?: string;
    theme?: Record<string, unknown>;
    sendDataModel?: boolean;
}

export interface PendingAction {
    actionId: string;
    surfaceId: string;
    name: string;
    responsePath?: string;
    createdAt: string;
}

export interface SurfaceState {
    components: Map<string, A2uiComponent>;
    dataModel: Record<string, unknown>;
    validationErrors: Map<string, string>;
    metadata: SurfaceMetadata;
}

export class A2uiStore {
    private surfaces = new Map<string, SurfaceState>();
    private pendingActions = new Map<string, PendingAction>();
    private lastCallFunctions: CallFunctionMessage[] = [];

    getSurface(surfaceId: string): SurfaceState | undefined {
        return this.surfaces.get(surfaceId);
    }

    getPendingAction(actionId: string): PendingAction | undefined {
        return this.pendingActions.get(actionId);
    }

    listPendingActions(): PendingAction[] {
        return Array.from(this.pendingActions.values());
    }

    trackPendingAction(action: PendingAction) {
        this.pendingActions.set(action.actionId, action);
    }

    clearPendingAction(actionId: string) {
        this.pendingActions.delete(actionId);
    }

    recordCallFunction(message: CallFunctionMessage) {
        this.lastCallFunctions = [message, ...this.lastCallFunctions].slice(0, 20);
    }

    listCallFunctions(): CallFunctionMessage[] {
        return [...this.lastCallFunctions];
    }

    ensureSurface(surfaceId: string): SurfaceState {
        const existing = this.surfaces.get(surfaceId);
        if (existing) {
            return existing;
        }
        const created: SurfaceState = {
            components: new Map(),
            dataModel: {},
            validationErrors: new Map(),
            metadata: {},
        };
        this.surfaces.set(surfaceId, created);
        return created;
    }

    /**
     * Apply an A2UI v1.0 actionResponse: write value to responsePath and clear pending state.
     */
    applyActionResponse(message: A2uiActionResponsePayload): PendingAction | undefined {
        const pending = this.pendingActions.get(message.actionId);
        if (!pending) {
            return undefined;
        }
        const surfaceId = pending.surfaceId;
        const surface = this.ensureSurface(surfaceId);
        const path = pending.responsePath ?? '/__a2ui/lastActionResponse';

        if (message.actionResponse.error) {
            this.setValidationError(
                surfaceId,
                path,
                `${message.actionResponse.error.code}: ${message.actionResponse.error.message}`,
            );
            surface.dataModel = writeJsonPointer(surface.dataModel, path, {
                error: message.actionResponse.error,
            });
        } else {
            this.clearValidationError(surfaceId, path);
            surface.dataModel = writeJsonPointer(
                surface.dataModel,
                path,
                message.actionResponse.value ?? null,
            );
        }

        this.pendingActions.delete(message.actionId);
        return pending;
    }

    configureSurface(surfaceId: string, metadata: SurfaceMetadata) {
        const surface = this.ensureSurface(surfaceId);
        surface.metadata = {
            ...surface.metadata,
            ...metadata,
        };
    }

    applyUpdateComponents(surfaceId: string, components: A2uiComponent[]) {
        const surface = this.ensureSurface(surfaceId);
        const FORBIDDEN_KEYS = new Set(["__proto__", "constructor", "prototype"]);
        for (const component of components) {
            if (!component.id || FORBIDDEN_KEYS.has(component.id)) {
                continue;
            }
            surface.components.set(component.id, component);
        }
    }

    replaceSurface(surfaceId: string, components: A2uiComponent[], dataModel: Record<string, unknown> = {}) {
        const surface = this.ensureSurface(surfaceId);
        surface.components.clear();
        surface.dataModel = dataModel;
        surface.validationErrors.clear();
        this.applyUpdateComponents(surfaceId, components);
    }

    removeSurface(surfaceId: string) {
        this.surfaces.delete(surfaceId);
    }

    setValidationError(surfaceId: string, path: string, message: string) {
        const surface = this.ensureSurface(surfaceId);
        surface.validationErrors.set(path, message);
    }

    clearValidationError(surfaceId: string, path: string) {
        const surface = this.ensureSurface(surfaceId);
        surface.validationErrors.delete(path);
    }

    applyUpdateDataModel(surfaceId: string, path: string | undefined, value: unknown) {
        const surface = this.ensureSurface(surfaceId);
        if (!path || path === "/") {
            surface.dataModel = (value as Record<string, unknown>) ?? {};
            return;
        }

        const tokens = path.split("/").filter(Boolean);
        if (tokens.length === 0) {
            surface.dataModel = (value as Record<string, unknown>) ?? {};
            return;
        }

        // Reject prototype-polluting keys
        const FORBIDDEN_KEYS = new Set(["__proto__", "constructor", "prototype"]);
        function isSafeKey(k: string): boolean {
            return !FORBIDDEN_KEYS.has(k);
        }

        let cursor: Record<string, unknown> = surface.dataModel;
        for (let i = 0; i < tokens.length - 1; i += 1) {
            const key = tokens[i];
            if (!isSafeKey(key)) {
                return;
            }
            const next = Object.prototype.hasOwnProperty.call(cursor, key) ? cursor[key] : undefined;
            if (typeof next === "object" && next !== null) {
                cursor = next as Record<string, unknown>;
            } else {
                const created: Record<string, unknown> = Object.create(null);
                Object.defineProperty(cursor, key, { value: created, writable: true, enumerable: true, configurable: true });
                cursor = created;
            }
        }
        const lastKey = tokens[tokens.length - 1];
        if (!isSafeKey(lastKey)) {
            return;
        }
        if (typeof value === "undefined") {
            delete cursor[lastKey];
        } else {
            Object.defineProperty(cursor, lastKey, { value: value, writable: true, enumerable: true, configurable: true });
        }
    }
}

const FORBIDDEN_POINTER_KEYS = new Set(['__proto__', 'constructor', 'prototype']);

/** Immutable JSON Pointer write used by actionResponse handling. */
export function writeJsonPointer(
    root: Record<string, unknown>,
    path: string,
    value: unknown,
): Record<string, unknown> {
    if (!path || path === '/') {
        return (value && typeof value === 'object' && !Array.isArray(value)
            ? { ...(value as Record<string, unknown>) }
            : { value }) as Record<string, unknown>;
    }

    const tokens = path
        .replace(/^\//, '')
        .split('/')
        .filter(Boolean)
        .map((token) => token.replace(/~1/g, '/').replace(/~0/g, '~'));

    if (tokens.length === 0 || tokens.some((token) => FORBIDDEN_POINTER_KEYS.has(token))) {
        return root;
    }

    const clone = typeof structuredClone === 'function'
        ? structuredClone(root)
        : (JSON.parse(JSON.stringify(root)) as Record<string, unknown>);

    let cursor: Record<string, unknown> = clone;
    for (let i = 0; i < tokens.length - 1; i += 1) {
        const key = tokens[i];
        const next = cursor[key];
        if (typeof next === 'object' && next !== null && !Array.isArray(next)) {
            cursor = next as Record<string, unknown>;
        } else {
            const created: Record<string, unknown> = {};
            cursor[key] = created;
            cursor = created;
        }
    }
    cursor[tokens[tokens.length - 1]] = value;
    return clone;
}
