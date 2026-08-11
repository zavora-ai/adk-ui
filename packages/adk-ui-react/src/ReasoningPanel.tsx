import React, { useMemo, useState } from 'react';
import clsx from 'clsx';

export type ReasoningMessage = {
    messageId: string;
    text: string;
    status: 'streaming' | 'complete';
};

export type ReasoningPanelProps = {
    messages: ReasoningMessage[];
    title?: string;
    className?: string;
    defaultOpen?: boolean;
    emptyLabel?: string;
};

/**
 * Collapsible panel for AG-UI REASONING_* event streams.
 */
export const ReasoningPanel: React.FC<ReasoningPanelProps> = ({
    messages,
    title = 'Agent reasoning',
    className,
    defaultOpen = true,
    emptyLabel = 'No reasoning captured for this run yet.',
}) => {
    const [open, setOpen] = useState(defaultOpen);
    const streaming = useMemo(
        () => messages.some((message) => message.status === 'streaming'),
        [messages],
    );

    return (
        <section
            className={clsx('adk-reasoning-panel', className)}
            aria-label={title}
            data-streaming={streaming ? 'true' : 'false'}
        >
            <button
                type="button"
                className="adk-reasoning-toggle"
                aria-expanded={open}
                onClick={() => setOpen((value) => !value)}
            >
                <span className="adk-reasoning-title">{title}</span>
                <span className="adk-reasoning-meta">
                    {streaming ? 'thinking…' : `${messages.length} step${messages.length === 1 ? '' : 's'}`}
                </span>
                <span className="adk-reasoning-chevron" aria-hidden="true">
                    {open ? '▾' : '▸'}
                </span>
            </button>
            {open ? (
                <div className="adk-reasoning-body">
                    {messages.length === 0 ? (
                        <p className="adk-reasoning-empty">{emptyLabel}</p>
                    ) : (
                        messages.map((message) => (
                            <article
                                key={message.messageId}
                                className={clsx(
                                    'adk-reasoning-item',
                                    message.status === 'streaming' && 'is-streaming',
                                )}
                            >
                                <header className="adk-reasoning-item-header">
                                    <span className="adk-reasoning-id">{message.messageId}</span>
                                    <span className="adk-reasoning-status">{message.status}</span>
                                </header>
                                <pre className="adk-reasoning-text">{message.text || '…'}</pre>
                            </article>
                        ))
                    )}
                </div>
            ) : null}
        </section>
    );
};

export type ReasoningRuntime = {
    messages: Map<string, ReasoningMessage>;
};

export function createReasoningRuntime(): ReasoningRuntime {
    return { messages: new Map() };
}

export function applyReasoningEvent(
    runtime: ReasoningRuntime,
    event: Record<string, unknown>,
): ReasoningMessage[] {
    const type = typeof event.type === 'string' ? event.type.trim().toUpperCase() : '';
    const messageIdRaw = event.messageId ?? event.message_id;
    const messageId = typeof messageIdRaw === 'string' ? messageIdRaw : undefined;
    const deltaRaw = event.delta ?? event.content;
    const delta = typeof deltaRaw === 'string' ? deltaRaw : undefined;
    const isReasoningEvent = type.startsWith('REASONING_') || type.startsWith('THINKING_');

    // AG-UI implicitly closes active reasoning when another event family begins.
    if (type && !isReasoningEvent) {
        completeStreamingMessages(runtime);
        return Array.from(runtime.messages.values());
    }

    if (type === 'REASONING_START' || type === 'REASONING_MESSAGE_START') {
        if (messageId) {
            const existing = runtime.messages.get(messageId);
            runtime.messages.set(messageId, {
                messageId,
                text: existing?.text ?? '',
                status: 'streaming',
            });
        }
    } else if (
        type === 'REASONING_MESSAGE_CONTENT'
        || type === 'REASONING_MESSAGE_CHUNK'
        || type === 'THINKING_TEXT_MESSAGE_CONTENT'
    ) {
        const keys = Array.from(runtime.messages.keys());
        const id = messageId ?? keys[keys.length - 1];
        if (id) {
            const existing = runtime.messages.get(id) ?? {
                messageId: id,
                text: '',
                status: 'streaming' as const,
            };
            const closesChunk = type === 'REASONING_MESSAGE_CHUNK' && delta === '';
            runtime.messages.set(id, closesChunk
                ? { ...existing, status: 'complete' }
                : {
                    ...existing,
                    text: `${existing.text}${delta ?? ''}`,
                    status: 'streaming',
                });
        }
    } else if (
        type === 'REASONING_MESSAGE_END'
        || type === 'REASONING_END'
        || type === 'THINKING_TEXT_MESSAGE_END'
        || type === 'THINKING_END'
    ) {
        if (messageId && runtime.messages.has(messageId)) {
            const existing = runtime.messages.get(messageId)!;
            runtime.messages.set(messageId, { ...existing, status: 'complete' });
        } else {
            completeStreamingMessages(runtime);
        }
    }

    return Array.from(runtime.messages.values());
}

function completeStreamingMessages(runtime: ReasoningRuntime) {
    for (const [id, existing] of runtime.messages) {
        if (existing.status === 'streaming') {
            runtime.messages.set(id, { ...existing, status: 'complete' });
        }
    }
}
