import type { ApplicationShowcaseExample } from '../types';
import { commerceKit } from '../kits';
import { brandCommerceMeta } from '../manifest';

export const brandCommerceExample: ApplicationShowcaseExample = {
  ...brandCommerceMeta,
  kit: commerceKit,
  kind: 'application',
  application: {
    id: 'atelier-material-concierge',
    name: 'Atelier',
    kit_id: commerceKit.id,
    initial_route: '/',
    navigation: [{ label: 'Edit', route: '/' }, { label: 'Selection', route: '/selection' }],
    pages: [
      {
        id: 'edit',
        route: '/',
        title: 'Material edit',
        template: 'marketing',
        atmosphere: 'gradient',
        eyebrow: 'Autumn material study / limited production',
        hero: {
          title: 'Choose by how you live, not by a filter grid.',
          description: 'The agent turns intended use, climate, and care preferences into a short, explainable material edit.',
          alignment: 'editorial',
          actions: [{ type: 'button', label: 'Build my selection', action_id: 'navigate:/selection', variant: 'primary' }],
          visual: [{
            type: 'card',
            title: 'Vegetable-tanned weekender',
            description: 'Tuscan hide / hand-finished brass / edition of 80',
            content: [
              { type: 'text', content: 'A structured travel piece that develops a deep patina instead of hiding wear.', variant: 'body' },
              { type: 'key_value', pairs: [{ key: 'Weight', value: '1.8 kg' }, { key: 'Capacity', value: '38 L' }, { key: 'Care', value: 'Seasonal conditioning' }] },
            ],
            footer: [{ type: 'button', label: 'Reserve this edition', action_id: 'reserve_weekender', variant: 'primary' }],
          }],
        },
        regions: [{
          id: 'material-principles',
          title: 'Three materials, three different lives',
          description: 'The comparison is organized around trade-offs rather than feature volume.',
          layout: 'bento',
          columns: 3,
          components: [
            { type: 'card', title: 'Full-grain leather', description: 'Ages visibly and repairs well', content: [{ type: 'progress', value: 92, label: 'Longevity' }, { type: 'badge', label: 'Best for considered travel', variant: 'success' }] },
            { type: 'card', title: 'Waxed canvas', description: 'Light, weather-ready, informal', content: [{ type: 'progress', value: 84, label: 'Weather resistance' }] },
            { type: 'card', title: 'Recycled sailcloth', description: 'Technical and nearly maintenance-free', content: [{ type: 'progress', value: 96, label: 'Low maintenance' }] },
            { type: 'text', content: '**Brand kit in control**\n\nColor, typography, spacing, radius, chart palette, and approved assets resolve from a reviewed manifest. The agent selects structure and content; the host retains visual governance.', variant: 'body' },
          ],
        }],
      },
      {
        id: 'selection',
        route: '/selection',
        title: 'Build a considered selection',
        template: 'form',
        atmosphere: 'clean',
        eyebrow: 'Concierge questions / no infinite catalog',
        regions: [{
          id: 'concierge-input',
          title: 'Tell us how the piece will be used',
          layout: 'split',
          columns: 2,
          components: [
            {
              type: 'card',
              title: 'Use and environment',
              content: [
                { type: 'select', name: 'primary_use', label: 'Primary use', required: true, options: [{ label: 'Two-night travel', value: 'weekend' }, { label: 'Daily commute', value: 'commute' }, { label: 'Frequent flying', value: 'flight' }] },
                { type: 'multi_select', name: 'conditions', label: 'Common conditions', options: [{ label: 'Rain', value: 'rain' }, { label: 'Warm climate', value: 'warm' }, { label: 'Rough handling', value: 'rough' }] },
                { type: 'slider', name: 'care_preference', label: 'Care tolerance', min: 0, max: 10, default_value: 4 },
              ],
              footer: [{ type: 'button', label: 'Generate recommendation', action_id: 'generate_material_recommendation', variant: 'primary' }],
            },
            {
              type: 'card',
              title: 'Likely recommendation',
              description: 'Based on the default two-night travel profile.',
              content: [
                { type: 'badge', label: 'Full-grain leather', variant: 'success' },
                { type: 'text', content: 'Best balance of structure, repairability, and long-term character. Choose waxed canvas if rain exposure becomes the dominant constraint.', variant: 'body' },
              ],
              footer: [{ type: 'button', label: 'Return to material edit', action_id: 'navigate:/', variant: 'outline' }],
            },
          ],
        }],
      },
    ],
  },
};
