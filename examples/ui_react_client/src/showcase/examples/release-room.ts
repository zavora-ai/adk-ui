import type { ApplicationShowcaseExample } from '../types';
import { releaseKit } from '../kits';
import { releaseRoomMeta } from '../manifest';

export const releaseRoomExample: ApplicationShowcaseExample = {
  ...releaseRoomMeta,
  kit: releaseKit,
  kind: 'application',
  application: {
    id: 'signal-release-room',
    name: 'Signal',
    kit_id: releaseKit.id,
    initial_route: '/',
    navigation: [
      { label: 'Briefing', route: '/' },
      { label: 'Rollout', route: '/rollout' },
      { label: 'Decision', route: '/decision' },
    ],
    state: { release: 'Mobile 8.4', rollout: 15 },
    pages: [
      {
        id: 'briefing',
        route: '/',
        title: 'Mobile 8.4 release briefing',
        template: 'marketing',
        atmosphere: 'gradient',
        eyebrow: 'Launch day / candidate 8.4.0-rc3',
        hero: {
          title: 'Ship the release you can explain.',
          description: 'One generated application connects launch posture, rollout controls, evidence, and the final human decision without flattening the workflow into a dashboard.',
          alignment: 'split',
          content: [{ type: 'badge', label: 'Conditional go', variant: 'warning' }],
          actions: [
            { type: 'button', label: 'Configure rollout', action_id: 'navigate:/rollout', variant: 'primary' },
            { type: 'button', label: 'Review decision', action_id: 'navigate:/decision', variant: 'outline' },
          ],
          visual: [{
            type: 'card',
            title: 'Readiness index',
            description: 'Weighted evidence, not a vanity score',
            content: [
              { type: 'progress', value: 87, label: '87 / 100 ready' },
              { type: 'key_value', pairs: [{ key: 'Automated checks', value: '428 / 428' }, { key: 'Open blockers', value: '2 conditional' }, { key: 'Crash-free beta', value: '99.82%' }] },
            ],
          }],
        },
        regions: [{
          id: 'release-signals',
          title: 'Signals that matter before launch',
          layout: 'bento',
          columns: 3,
          components: [
            { type: 'card', title: 'Beta quality improved', description: 'Seven-day crash-free sessions', content: [{ type: 'chart', kind: 'area', x_key: 'day', y_keys: ['rate'], data: [{ day: 'Mon', rate: 99.61 }, { day: 'Tue', rate: 99.68 }, { day: 'Wed', rate: 99.74 }, { day: 'Thu', rate: 99.79 }, { day: 'Fri', rate: 99.82 }] }] },
            { type: 'card', title: '428', description: 'Passing checks', content: [{ type: 'badge', label: 'No regressions', variant: 'success' }] },
            { type: 'card', title: '2', description: 'Conditional blockers', content: [{ type: 'badge', label: 'Owned', variant: 'warning' }] },
            { type: 'alert', title: 'Watch checkout authentication', description: 'A 0.3% retry increase appears limited to iOS 19 beta devices.', variant: 'warning' },
          ],
        }],
      },
      {
        id: 'rollout',
        route: '/rollout',
        title: 'Progressive rollout plan',
        description: 'Edit the generated proposal. Navigate away and return: this page remains mounted so local form values survive.',
        template: 'workspace',
        atmosphere: 'grid',
        eyebrow: 'Editable operating plan',
        regions: [
          {
            id: 'rollout-plan',
            title: 'Release stages',
            layout: 'split',
            columns: 2,
            components: [
              {
                type: 'card',
                title: 'Rollout controls',
                description: 'Agent proposal, operator editable.',
                content: [
                  { type: 'slider', name: 'initial_rollout', label: 'Initial audience (%)', min: 1, max: 50, step: 1, default_value: 15 },
                  { type: 'select', name: 'first_region', label: 'First region', required: true, options: [{ label: 'Canada', value: 'ca' }, { label: 'United Kingdom', value: 'uk' }, { label: 'Australia', value: 'au' }] },
                  { type: 'number_input', name: 'observation_window', label: 'Observation window (minutes)', min: 15, max: 180, default_value: 45 },
                  { type: 'switch', name: 'automatic_pause', label: 'Pause automatically on SLO breach', default_checked: true },
                ],
                footer: [{ type: 'button', label: 'Save rollout plan', action_id: 'save_rollout_plan', variant: 'primary' }],
              },
              {
                type: 'table',
                columns: [{ header: 'Stage', accessor_key: 'stage' }, { header: 'Audience', accessor_key: 'audience' }, { header: 'Advance when', accessor_key: 'gate' }],
                data: [
                  { stage: 'Canary', audience: '15%', gate: '45m healthy' },
                  { stage: 'Expand', audience: '40%', gate: '2h healthy' },
                  { stage: 'General', audience: '100%', gate: 'Owner approval' },
                ],
                striped: true,
              },
            ],
          },
          {
            id: 'rollback-contract',
            title: 'Automatic stop conditions',
            layout: 'stats',
            columns: 3,
            tone: 'muted',
            components: [
              { type: 'card', title: '> 0.8%', description: 'Crash rate', content: [{ type: 'badge', label: 'Hard stop', variant: 'error' }] },
              { type: 'card', title: '> 2.0%', description: 'Auth retry delta', content: [{ type: 'badge', label: 'Pause + review', variant: 'warning' }] },
              { type: 'card', title: '< 98.5%', description: 'Checkout success', content: [{ type: 'badge', label: 'Hard stop', variant: 'error' }] },
            ],
          },
        ],
        footer: [{ type: 'button', label: 'Continue to decision', action_id: 'navigate:/decision', variant: 'outline' }],
      },
      {
        id: 'decision',
        route: '/decision',
        title: 'Go / no-go decision',
        template: 'detail',
        atmosphere: 'clean',
        eyebrow: 'Human authority boundary',
        regions: [{
          id: 'decision-evidence',
          title: 'Decision record',
          description: 'The recommendation remains traceable to evidence and the editable rollout contract.',
          layout: 'editorial',
          columns: 2,
          components: [
            { type: 'key_value', pairs: [{ key: 'Recommendation', value: 'Go with staged rollout' }, { key: 'Confidence', value: '87%' }, { key: 'Initial audience', value: '15%' }, { key: 'Open condition', value: 'Watch iOS 19 auth retries' }] },
            { type: 'alert', title: 'Approval is not inferred', description: 'The agent can recommend and prepare the action. A release owner must explicitly approve it.', variant: 'info' },
          ],
        }],
        aside: [{
          type: 'card',
          title: 'Release owner decision',
          description: 'Captured as a structured UI event.',
          content: [
            { type: 'select', name: 'decision', label: 'Decision', required: true, options: [{ label: 'Approve staged rollout', value: 'approve' }, { label: 'Hold release', value: 'hold' }, { label: 'Return for changes', value: 'revise' }] },
            { type: 'textarea', name: 'decision_rationale', label: 'Rationale', required: true, rows: 4, placeholder: 'State the evidence behind this decision' },
          ],
          footer: [{ type: 'button', label: 'Record decision', action_id: 'record_release_decision', variant: 'primary' }],
        }],
      },
    ],
  },
};
