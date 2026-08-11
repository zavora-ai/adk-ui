const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    applyReasoningEvent,
    createReasoningRuntime,
    isActionResponseMessage,
    writeJsonPointer,
} = require('../dist/index.js');

describe('reasoning runtime', () => {
    it('accumulates REASONING_MESSAGE_CONTENT chunks', () => {
        const runtime = createReasoningRuntime();
        applyReasoningEvent(runtime, {
            type: 'REASONING_START',
            messageId: 'r1',
        });
        applyReasoningEvent(runtime, {
            type: 'REASONING_MESSAGE_CONTENT',
            messageId: 'r1',
            delta: 'I will ',
        });
        const messages = applyReasoningEvent(runtime, {
            type: 'REASONING_MESSAGE_CONTENT',
            messageId: 'r1',
            delta: 'confirm next.',
        });
        assert.equal(messages.length, 1);
        assert.equal(messages[0].text, 'I will confirm next.');
        assert.equal(messages[0].status, 'streaming');

        const done = applyReasoningEvent(runtime, {
            type: 'REASONING_END',
            messageId: 'r1',
        });
        assert.equal(done[0].status, 'complete');
    });

    it('closes an empty chunk and on the next non-reasoning event', () => {
        const runtime = createReasoningRuntime();
        applyReasoningEvent(runtime, {
            type: 'REASONING_MESSAGE_CHUNK',
            messageId: 'r1',
            delta: 'Checking',
        });
        const closedByChunk = applyReasoningEvent(runtime, {
            type: 'REASONING_MESSAGE_CHUNK',
            messageId: 'r1',
            delta: '',
        });
        assert.equal(closedByChunk[0].status, 'complete');

        applyReasoningEvent(runtime, {
            type: 'REASONING_MESSAGE_CHUNK',
            messageId: 'r2',
            delta: 'Continuing',
        });
        const closedByBoundary = applyReasoningEvent(runtime, {
            type: 'TEXT_MESSAGE_START',
            messageId: 'answer-1',
        });
        assert.equal(closedByBoundary[1].status, 'complete');
    });
});

describe('actionResponse helpers', () => {
    it('detects actionResponse envelopes', () => {
        assert.equal(
            isActionResponseMessage({
                version: 'v1.0',
                actionId: 'act-1',
                actionResponse: { value: ['a'] },
            }),
            true,
        );
        assert.equal(
            isActionResponseMessage({
                createSurface: { surfaceId: 'main' },
            }),
            false,
        );
    });

    it('writes response values at JSON pointers', () => {
        const next = writeJsonPointer(
            { form: { email: 'a@b.c' } },
            '/form/suggestions',
            ['alice', 'alex'],
        );
        assert.equal(next.form.email, 'a@b.c');
        assert.deepEqual(next.form.suggestions, ['alice', 'alex']);
    });
});
