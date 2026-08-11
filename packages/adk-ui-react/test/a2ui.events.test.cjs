const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const {
    buildActionEvent,
    buildErrorEvent,
    buildValidationFailedEvent,
    resolveActionResponsePath,
    runLocalAction,
} = require('../dist/index.js');

describe('A2ui events', () => {
    it('builds action events with resolved context', () => {
        const event = buildActionEvent(
            {
                event: {
                    name: 'submitForm',
                    context: {
                        userId: { path: '/user/id' },
                        literal: 'ok',
                        score: { call: 'add', args: [1, { path: '/score' }] },
                    },
                },
            },
            'main',
            'submit_button',
            {
                dataModel: { user: { id: 'u-1' }, score: 2 },
                timestamp: new Date('2026-01-25T12:00:00Z'),
            },
        );

        assert.equal(event.action.name, 'submitForm');
        assert.equal(event.action.surfaceId, 'main');
        assert.equal(event.action.sourceComponentId, 'submit_button');
        assert.equal(event.action.timestamp, '2026-01-25T12:00:00.000Z');
        assert.deepEqual(event.action.context, {
            userId: 'u-1',
            literal: 'ok',
            score: 3,
        });
    });

    it('assigns actionId when wantResponse is set (A2UI v1.0)', () => {
        const event = buildActionEvent(
            {
                event: {
                    name: 'typeahead',
                    wantResponse: true,
                    responsePath: '/form/suggestions',
                    context: { prefix: { path: '/form/q' } },
                },
            },
            'main',
            'input_1',
            {
                dataModel: { form: { q: 'app' } },
            },
        );

        assert.equal(event.version, 'v1.0');
        assert.equal(event.action.wantResponse, true);
        assert.equal('responsePath' in event.action, false);
        assert.equal(
            resolveActionResponsePath({
                event: {
                    name: 'typeahead',
                    wantResponse: true,
                    responsePath: '/form/suggestions',
                },
            }),
            '/form/suggestions',
        );
        assert.equal(typeof event.action.actionId, 'string');
        assert.match(event.action.actionId, /^typeahead_/);
        assert.equal(event.action.context.prefix, 'app');
    });

    it('returns null when no event is configured', () => {
        const event = buildActionEvent(undefined, 'main', 'btn', { dataModel: {} });
        assert.equal(event, null);
    });

    it('builds validation and generic error payloads', () => {
        const validation = buildValidationFailedEvent('main', '/form/email', 'Email is required.');
        const generic = buildErrorEvent('UNKNOWN_COMPONENT', 'main', 'Unsupported component.', {
            componentId: 'legacy-1',
        });

        assert.deepEqual(validation, {
            error: {
                code: 'VALIDATION_FAILED',
                surfaceId: 'main',
                path: '/form/email',
                message: 'Email is required.',
            },
        });
        assert.deepEqual(generic, {
            error: {
                code: 'UNKNOWN_COMPONENT',
                surfaceId: 'main',
                message: 'Unsupported component.',
                componentId: 'legacy-1',
            },
        });
    });

    it('runs local openUrl actions through the provided handler', () => {
        let opened = null;
        const result = runLocalAction(
            {
                functionCall: {
                    call: 'openUrl',
                    args: { url: 'https://example.com/docs' },
                },
            },
            {
                dataModel: {},
                openUrl: (url) => {
                    opened = url;
                },
            },
        );

        assert.equal(result, 'https://example.com/docs');
        assert.equal(opened, 'https://example.com/docs');
    });
});
