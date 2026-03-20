import type { FunctionCall, FunctionRegistry } from './bindings';
import { resolveDynamicValue } from './bindings';
import { sanitizeUrl } from '../security';

export type A2uiActionEventDefinition = {
    name: string;
    context?: Record<string, unknown>;
};

export type A2uiActionDefinition = {
    event?: A2uiActionEventDefinition;
    functionCall?: FunctionCall;
};

export type A2uiActionEventPayload = {
    action: {
        name: string;
        surfaceId: string;
        sourceComponentId: string;
        timestamp: string;
        context: Record<string, unknown>;
    };
};

export type A2uiValidationErrorPayload = {
    error: {
        code: 'VALIDATION_FAILED';
        surfaceId: string;
        path: string;
        message: string;
    };
};

export type A2uiGenericErrorPayload = {
    error: {
        code: string;
        surfaceId: string;
        message: string;
    } & Record<string, unknown>;
};

export type A2uiClientMessagePayload =
    | A2uiActionEventPayload
    | A2uiValidationErrorPayload
    | A2uiGenericErrorPayload;

export interface ActionEventOptions {
    dataModel: Record<string, unknown>;
    scope?: Record<string, unknown>;
    functions?: FunctionRegistry;
    timestamp?: Date;
}

export function buildActionEvent(
    action: A2uiActionDefinition | undefined,
    surfaceId: string,
    sourceComponentId: string,
    options: ActionEventOptions,
): A2uiActionEventPayload | null {
    if (!action?.event?.name) {
        return null;
    }
    const context = resolveActionContext(action.event.context ?? {}, options);
    return {
        action: {
            name: action.event.name,
            surfaceId,
            sourceComponentId,
            timestamp: (options.timestamp ?? new Date()).toISOString(),
            context,
        },
    };
}

export function buildValidationFailedEvent(
    surfaceId: string,
    path: string,
    message: string,
): A2uiValidationErrorPayload {
    return {
        error: {
            code: 'VALIDATION_FAILED',
            surfaceId,
            path,
            message,
        },
    };
}

export function buildErrorEvent(
    code: string,
    surfaceId: string,
    message: string,
    details: Record<string, unknown> = {},
): A2uiGenericErrorPayload {
    return {
        error: {
            code,
            surfaceId,
            message,
            ...details,
        },
    };
}

export function runLocalAction(
    action: A2uiActionDefinition | undefined,
    options: ActionEventOptions & { openUrl?: (url: string) => void },
): unknown {
    if (!action?.functionCall) {
        return undefined;
    }

    const result = resolveDynamicValue(
        action.functionCall,
        options.dataModel,
        options.scope,
        options.functions,
    );

    if (action.functionCall.call === 'openUrl' && typeof result === 'string') {
        const safeUrl = sanitizeUrl(result, 'anchor');
        if (safeUrl) {
            options.openUrl?.(safeUrl);
        }
    }

    return result;
}

function resolveActionContext(
    context: Record<string, unknown>,
    options: ActionEventOptions,
) {
    const resolved: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(context)) {
        resolved[key] = resolveDynamicValue(
            value,
            options.dataModel,
            options.scope,
            options.functions,
        );
    }
    return resolved;
}
