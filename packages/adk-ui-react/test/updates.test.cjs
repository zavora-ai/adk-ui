const { describe, it } = require('node:test');
const assert = require('node:assert/strict');

const { applyUiUpdate, applyUiUpdates } = require('../dist/index.js');

describe('applyUiUpdate', () => {
    it('replaces a component by id', () => {
        const component = { type: 'text', id: 'root', content: 'hello' };
        const update = {
            target_id: 'root',
            operation: 'replace',
            payload: { type: 'text', id: 'root', content: 'updated' },
        };
        const updated = applyUiUpdate(component, update);
        assert.equal(updated.content, 'updated');
    });

    it('patches a nested component', () => {
        const component = {
            type: 'stack',
            id: 'root',
            direction: 'vertical',
            children: [
                { type: 'text', id: 'title', content: 'Title', variant: 'h1' },
            ],
        };
        const update = {
            target_id: 'title',
            operation: 'patch',
            payload: { type: 'text', id: 'title', content: 'New Title' },
        };
        const updated = applyUiUpdate(component, update);
        assert.equal(updated.children[0].content, 'New Title');
    });

    it('appends into containers', () => {
        const component = {
            type: 'stack',
            id: 'root',
            direction: 'vertical',
            children: [],
        };
        const update = {
            target_id: 'root',
            operation: 'append',
            payload: { type: 'text', id: 'child', content: 'Added' },
        };
        const updated = applyUiUpdate(component, update);
        assert.equal(updated.children.length, 1);
        assert.equal(updated.children[0].id, 'child');
    });

    it('removes a component by id', () => {
        const component = {
            type: 'stack',
            id: 'root',
            direction: 'vertical',
            children: [
                { type: 'text', id: 'to-remove', content: 'Remove' },
                { type: 'text', id: 'keep', content: 'Keep' },
            ],
        };
        const update = {
            target_id: 'to-remove',
            operation: 'remove',
        };
        const updated = applyUiUpdate(component, update);
        assert.equal(updated.children.length, 1);
        assert.equal(updated.children[0].id, 'keep');
    });
});

describe('applyUiUpdates', () => {
    it('applies multiple updates in order', () => {
        const component = { type: 'text', id: 'root', content: 'hello' };
        const updates = [
            { target_id: 'root', operation: 'patch', payload: { type: 'text', id: 'root', content: 'patched' } },
            { target_id: 'root', operation: 'replace', payload: { type: 'text', id: 'root', content: 'replaced' } },
        ];
        const updated = applyUiUpdates(component, updates);
        assert.equal(updated.content, 'replaced');
    });
});
