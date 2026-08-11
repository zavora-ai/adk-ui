import type { A2uiComponent, A2uiStore } from './store';
import type { A2uiActionResponsePayload } from './events';

export type A2uiProtocolVersion = 'v0.9' | 'v0.9.1' | 'v1.0';

export interface A2uiMessageBase {
    version?: A2uiProtocolVersion | string;
}

export type CreateSurfaceMessage = A2uiMessageBase & {
    createSurface: {
        surfaceId: string;
        catalogId?: string;
        theme?: Record<string, unknown>;
        sendDataModel?: boolean;
        components?: A2uiComponent[];
        dataModel?: Record<string, unknown>;
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

export type ActionResponseMessage = A2uiMessageBase & A2uiActionResponsePayload;

export type CallFunctionMessage = A2uiMessageBase & {
    functionCallId: string;
    wantResponse?: boolean;
    callFunction: {
        call: string;
        args?: Record<string, unknown>;
    };
};

export type A2uiMessage =
    | CreateSurfaceMessage
    | UpdateComponentsMessage
    | UpdateDataModelMessage
    | DeleteSurfaceMessage
    | ActionResponseMessage
    | CallFunctionMessage;

export interface ParsedA2uiMessage {
    message: A2uiMessage;
    raw: string;
}

export function isActionResponseMessage(
    message: A2uiMessage,
): message is ActionResponseMessage {
    return (
        'actionId' in message
        && 'actionResponse' in message
        && typeof (message as ActionResponseMessage).actionId === 'string'
    );
}

export function isCallFunctionMessage(
    message: A2uiMessage,
): message is CallFunctionMessage {
    return (
        'functionCallId' in message
        && 'callFunction' in message
    );
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
            if (message.createSurface.components?.length) {
                store.applyUpdateComponents(
                    message.createSurface.surfaceId,
                    message.createSurface.components,
                );
            }
            if (message.createSurface.dataModel) {
                store.applyUpdateDataModel(
                    message.createSurface.surfaceId,
                    '/',
                    message.createSurface.dataModel,
                );
            }
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
        } else if (isActionResponseMessage(message)) {
            store.applyActionResponse(message);
        } else if (isCallFunctionMessage(message)) {
            store.recordCallFunction(message);
        }
    }
}
