import type { ApplicationShowcaseExample } from '../types';
import { incidentKit } from '../kits';
import { incidentCommandMeta } from '../manifest';

export const incidentCommandExample: ApplicationShowcaseExample = {
  ...incidentCommandMeta,
  kit: incidentKit,
  kind: 'application',
  application: {
    id: 'northstar-incident-command',
    name: 'Northstar',
    kit_id: incidentKit.id,
    initial_route: '/',
    navigation: [{ label: 'Command', route: '/' }],
    state: { incidentId: 'INC-4821', region: 'us-east-1' },
    pages: [{
      id: 'command',
      route: '/',
      title: 'Checkout latency incident',
      description: 'Correlated signals and a reversible mitigation plan for INC-4821.',
      template: 'dashboard',
      atmosphere: 'grid',
      eyebrow: 'SEV-1 / 18:42 UTC / us-east-1',
      regions: [
        {
          id: 'incident-status',
          layout: 'stats',
          columns: 4,
          components: [
            { type: 'card', title: '2.8s', description: 'p95 checkout latency', content: [{ type: 'badge', label: '+164% vs baseline', variant: 'error' }] },
            { type: 'card', title: '7.4%', description: 'Payment failures', content: [{ type: 'badge', label: '12,840 sessions', variant: 'warning' }] },
            { type: 'card', title: '18 min', description: 'Incident duration', content: [{ type: 'progress', value: 62, label: 'Error budget consumed' }] },
            { type: 'card', title: '86%', description: 'Rollback confidence', content: [{ type: 'badge', label: '3 signals agree', variant: 'success' }] },
          ],
        },
        {
          id: 'incident-evidence',
          title: 'Evidence before action',
          description: 'The agent correlates the deploy, regional impact, and recovery estimate instead of presenting an unqualified recommendation.',
          layout: 'split',
          columns: 2,
          components: [
            {
              type: 'chart',
              title: 'Latency and failed checkouts',
              kind: 'area',
              x_key: 'time',
              y_keys: ['latency', 'failures'],
              show_legend: true,
              data: [
                { time: '18:20', latency: 0.9, failures: 1.2 },
                { time: '18:25', latency: 1.0, failures: 1.3 },
                { time: '18:30', latency: 1.8, failures: 3.6 },
                { time: '18:35', latency: 2.6, failures: 6.8 },
                { time: '18:40', latency: 2.8, failures: 7.4 },
              ],
            },
            {
              type: 'table',
              sortable: true,
              striped: true,
              page_size: 4,
              columns: [
                { header: 'Signal', accessor_key: 'signal' },
                { header: 'Evidence', accessor_key: 'evidence' },
                { header: 'Confidence', accessor_key: 'confidence' },
              ],
              data: [
                { signal: 'Deploy', evidence: 'payments-api v4.18 at 18:27', confidence: 'High' },
                { signal: 'Region', evidence: '91% of failures in us-east-1', confidence: 'High' },
                { signal: 'Dependency', evidence: 'Token service remains healthy', confidence: 'High' },
                { signal: 'Recovery', evidence: 'Previous rollback recovered in 6m', confidence: 'Medium' },
              ],
            },
          ],
        },
        {
          id: 'incident-timeline',
          title: 'Operator timeline',
          layout: 'editorial',
          columns: 2,
          tone: 'muted',
          components: [
            { type: 'text', content: '**18:27** Release v4.18 completed in us-east-1.\n\n**18:31** Checkout SLO breached.\n\n**18:36** Traffic shift ruled out as primary cause.', variant: 'body' },
            { type: 'alert', title: 'Recommendation', description: 'Rollback payments-api in us-east-1 only, hold other regions, and verify error rate for five minutes.', variant: 'warning' },
          ],
        },
      ],
      aside: [{
        type: 'card',
        title: 'Approve with edits',
        description: 'The agent proposes scope. The operator owns the decision.',
        content: [
          { type: 'select', name: 'rollback_scope', label: 'Rollback scope', required: true, options: [{ label: 'us-east-1 only', value: 'us-east-1' }, { label: 'All regions', value: 'all' }] },
          { type: 'slider', name: 'verification_window', label: 'Verification window (minutes)', min: 3, max: 15, step: 1, default_value: 5 },
          { type: 'textarea', name: 'operator_note', label: 'Decision note', rows: 3, default_value: 'Scope limited to affected region. Hold if token service degrades.' },
          { type: 'switch', name: 'notify_customers', label: 'Prepare customer update', default_checked: true },
        ],
        footer: [
          { type: 'button', label: 'Approve rollback', action_id: 'approve_regional_rollback', variant: 'danger' },
          { type: 'button', label: 'Keep investigating', action_id: 'defer_rollback', variant: 'outline' },
        ],
      }],
      footer: [{ type: 'text', content: 'Inputs are illustrative. No production action is executed by this showcase.', variant: 'caption' }],
    }],
  },
};
