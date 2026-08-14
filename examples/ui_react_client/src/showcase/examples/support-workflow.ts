import type { ApplicationShowcaseExample } from '../types';
import { supportKit } from '../kits';
import { supportWorkflowMeta } from '../manifest';

export const supportWorkflowExample: ApplicationShowcaseExample = {
  ...supportWorkflowMeta,
  kit: supportKit,
  kind: 'application',
  application: {
    id: 'harbor-guided-support',
    name: 'Harbor',
    kit_id: supportKit.id,
    initial_route: '/',
    navigation: [{ label: 'Intake', route: '/' }, { label: 'Review', route: '/review' }],
    state: { severity: 'high', environment: 'production' },
    pages: [
      {
        id: 'intake',
        route: '/',
        title: 'Payroll export escalation',
        template: 'form',
        atmosphere: 'gradient',
        eyebrow: 'Step 1 of 2 / response due in 14 minutes',
        hero: {
          title: 'Let us collect what the responder still needs.',
          description: 'The report, production environment, finance cutoff, and high severity are already known. This form does not ask for them again.',
          alignment: 'editorial',
          content: [{ type: 'progress', value: 50, label: 'Intake progress' }],
          visual: [{
            type: 'card',
            title: 'Known from your message',
            description: 'Preserved as agent state',
            content: [{
              type: 'key_value',
              pairs: [
                { key: 'Issue', value: 'Payroll export failed' },
                { key: 'Environment', value: 'Production' },
                { key: 'Deadline', value: 'Finance cutoff today' },
                { key: 'Severity', value: 'High' },
              ],
            }],
          }],
        },
        regions: [{
          id: 'missing-evidence',
          title: 'Complete the evidence',
          description: 'Four fields replace a generic support questionnaire.',
          layout: 'split',
          columns: 2,
          components: [
            {
              type: 'card',
              title: 'Impact and timing',
              description: 'Required to set queue priority.',
              content: [
                { type: 'number_input', name: 'affected_employees', label: 'Employees affected', required: true, min: 1, max: 100000, default_value: 640 },
                { type: 'select', name: 'workaround', label: 'Is a workaround available?', required: true, options: [{ label: 'No workaround', value: 'none' }, { label: 'Partial manual workaround', value: 'partial' }, { label: 'Full workaround', value: 'full' }] },
                { type: 'text_input', name: 'cutoff_time', label: 'Finance cutoff time', required: true, default_value: '17:00 UTC' },
              ],
              footer: [{ type: 'button', label: 'Save impact details', action_id: 'save_impact', variant: 'primary' }],
            },
            {
              type: 'card',
              title: 'Evidence and contact',
              description: 'Enough detail for the first responder to begin.',
              content: [
                { type: 'textarea', name: 'error_evidence', label: 'Exact error or log reference', required: true, rows: 4, placeholder: 'Paste the error text or an internal log link' },
                { type: 'text_input', name: 'contact', label: 'Responder contact', input_type: 'email', required: true, default_value: 'finance-ops@example.com' },
                { type: 'switch', name: 'safe_to_reproduce', label: 'Safe to reproduce in production', default_checked: false },
              ],
              footer: [{ type: 'button', label: 'Continue to review', action_id: 'navigate:/review', variant: 'primary' }],
            },
          ],
        }],
        aside: [{ type: 'alert', title: 'High-severity SLA', description: 'An on-call responder should acknowledge this incident within 15 minutes. Submission does not authorize a production change.', variant: 'info' }],
      },
      {
        id: 'review',
        route: '/review',
        title: 'Review escalation',
        template: 'detail',
        atmosphere: 'clean',
        eyebrow: 'Step 2 of 2 / human confirmation',
        regions: [
          {
            id: 'review-summary',
            title: 'What the responder will receive',
            layout: 'editorial',
            columns: 2,
            components: [
              { type: 'key_value', pairs: [{ key: 'Queue', value: 'Payroll production on-call' }, { key: 'Priority', value: 'P1 / finance cutoff risk' }, { key: 'Known impact', value: '640 employees' }, { key: 'Change authority', value: 'Not granted' }] },
              { type: 'alert', title: 'Nothing is escalated yet', description: 'Return to Intake to edit evidence, or confirm below to create the ticket.', variant: 'warning' },
            ],
          },
        ],
        aside: [{
          type: 'card',
          title: 'Final confirmation',
          description: 'You remain in control of the outbound action.',
          content: [{ type: 'switch', name: 'confirm_accuracy', label: 'I reviewed the escalation summary', default_checked: false }],
          footer: [
            { type: 'button', label: 'Create escalation', action_id: 'create_payroll_escalation', variant: 'primary' },
            { type: 'button', label: 'Edit intake', action_id: 'navigate:/', variant: 'outline' },
          ],
        }],
      },
    ],
  },
};
