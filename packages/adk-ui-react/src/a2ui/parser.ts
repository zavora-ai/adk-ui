import type { A2uiComponent, A2uiStore } from './store';

export type A2uiProtocolVersion = 'v0.9';

export interface A2uiMessageBase {
    version?: A2uiProtocolVersion;
}

export type CreateSurfaceMessage = A2uiMessageBase & {
    createSurface: {
        surfaceId: string;
        catalogId: string;
        theme?: Record<string, unknown>;
        sendDataModel?: boolean;
    };
};

export type UpdateComponentsMessage = A2uiMessageBase & {
    updateComponents: {
        surfaceId: string;
        components: A2uiComponent[];
    };
};

export type UpdateDataModelMessage = A2uiMessageBase & {
    updateDataModel: {
        surfaceId: string;
        path?: string;
        value?: unknown;
    };
};

export type DeleteSurfaceMessage = A2uiMessageBase & {
    deleteSurface: {
        surfaceId: string;
    };
};

export type A2uiMessage =
    | CreateSurfaceMessage
    | UpdateComponentsMessage
    | UpdateDataModelMessage
    | DeleteSurfaceMessage;

export interface ParsedA2uiMessage {
    message: A2uiMessage;
    raw: string;
}

export function parseJsonl(payload: string): ParsedA2uiMessage[] {
    return payload
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0)
        .map((line) => ({
            message: JSON.parse(line) as A2uiMessage,
            raw: line,
        }));
}

export function applyParsedMessages(store: A2uiStore, parsed: ParsedA2uiMessage[]) {
    for (const entry of parsed) {
        const message = entry.message;
        if ('createSurface' in message) {
            store.ensureSurface(message.createSurface.surfaceId);
            store.configureSurface(message.createSurface.surfaceId, {
                catalogId: message.createSurface.catalogId,
                theme: message.createSurface.theme,
                sendDataModel: message.createSurface.sendDataModel,
            });
        } else if ('updateComponents' in message) {
            store.applyUpdateComponents(
                message.updateComponents.surfaceId,
                message.updateComponents.components,
            );
        } else if ('updateDataModel' in message) {
            store.applyUpdateDataModel(
                message.updateDataModel.surfaceId,
                message.updateDataModel.path,
                message.updateDataModel.value,
            );
        } else if ('deleteSurface' in message) {
            store.removeSurface(message.deleteSurface.surfaceId);
        }
    }
}
