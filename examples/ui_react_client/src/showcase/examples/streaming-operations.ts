import type { StreamingShowcaseExample } from '../types';
import { streamingKit } from '../kits';
import { streamingOperationsMeta } from '../manifest';

export const streamingOperationsExample: StreamingShowcaseExample = {
  ...streamingOperationsMeta,
  kit: streamingKit,
  kind: 'streaming',
  initialComponent: {
    type: 'stack',
    id: 'stream-root',
    direction: 'vertical',
    gap: 18,
    children: [
      { type: 'text', id: 'stream-title', content: 'Treasury reconciliation', variant: 'h1' },
      { type: 'text', id: 'stream-copy', content: 'The agent is correlating bank feeds, ledger entries, and exception ownership.', variant: 'body' },
      { type: 'alert', id: 'stream-status', title: 'Connecting data sources', description: 'Waiting for the first reconciled batch.', variant: 'info' },
      {
        type: 'grid',
        id: 'stream-metrics',
        columns: 3,
        gap: 14,
        children: [
          { type: 'skeleton', id: 'metric-a', variant: 'rectangle', height: '132px' },
          { type: 'skeleton', id: 'metric-b', variant: 'rectangle', height: '132px' },
          { type: 'skeleton', id: 'metric-c', variant: 'rectangle', height: '132px' },
        ],
      },
      { type: 'stack', id: 'stream-timeline', direction: 'vertical', gap: 10, children: [] },
    ],
  },
  updates: [
    {
      target_id: 'stream-status',
      operation: 'replace',
      payload: { type: 'alert', id: 'stream-status', title: 'Bank feed reconciled', description: '2,418 entries matched. Ledger comparison is running.', variant: 'success' },
    },
    {
      target_id: 'stream-metrics',
      operation: 'replace',
      payload: {
        type: 'grid',
        id: 'stream-metrics',
        columns: 3,
        gap: 14,
        children: [
          { type: 'card', title: '$4.82m', description: 'Reconciled value', content: [{ type: 'badge', label: '99.2% matched', variant: 'success' }] },
          { type: 'card', title: '19', description: 'Open exceptions', content: [{ type: 'badge', label: '6 need owner', variant: 'warning' }] },
          { type: 'card', title: '3m 42s', description: 'Time recovered', content: [{ type: 'badge', label: 'Live run', variant: 'info' }] },
        ],
      },
    },
    {
      target_id: 'stream-timeline',
      operation: 'append',
      payload: { type: 'alert', id: 'exception-found', title: 'Material exception found', description: '$84,200 settlement differs from the ledger by $12,400.', variant: 'warning' },
    },
    {
      target_id: 'stream-timeline',
      operation: 'append',
      payload: {
        type: 'card',
        id: 'exception-action',
        title: 'Settlement ST-9924',
        description: 'Agent recommendation: assign to Treasury Operations with the bank trace attached.',
        content: [{ type: 'key_value', pairs: [{ key: 'Bank value', value: '$84,200' }, { key: 'Ledger value', value: '$71,800' }, { key: 'Difference', value: '$12,400' }] }],
        footer: [{ type: 'button', label: 'Assign exception', action_id: 'assign_settlement_exception', variant: 'primary' }],
      },
    },
    {
      target_id: 'stream-status',
      operation: 'patch',
      payload: { type: 'alert', id: 'stream-status', title: 'Reconciliation ready for review', description: 'The complete surface stayed interactive while targeted regions changed.', variant: 'success' },
    },
    {
      target_id: 'exception-found',
      operation: 'remove',
    },
  ],
};
