import type { ShowcaseExampleMeta } from './types';

export const incidentCommandMeta: ShowcaseExampleMeta = {
  id: 'incident-command',
  number: '01',
  title: 'Incident command',
  eyebrow: 'Decision workspace',
  description: 'A live operations surface that ranks evidence, exposes uncertainty, and keeps a risky rollback behind explicit human approval.',
  capabilitySummary: 'Dense data, charts, tables, editable action scope, and human approval.',
  capabilities: ['Decision support', 'Charts + tables', 'Guarded action', 'Responsive layout'],
  protocols: ['AG-UI', 'A2UI'],
  accent: '#ff5a36',
};

export const supportWorkflowMeta: ShowcaseExampleMeta = {
  id: 'support-workflow',
  number: '02',
  title: 'Guided support',
  eyebrow: 'Adaptive form',
  description: 'A calm intake that preserves known context, asks only for missing evidence, explains urgency, and creates an editable review boundary.',
  capabilitySummary: 'Structured input, native validation, progress, context preservation, and confirmation.',
  capabilities: ['Adaptive forms', 'Validation', 'Known-state reuse', 'Review before submit'],
  protocols: ['A2UI'],
  accent: '#0a6b58',
};

export const releaseRoomMeta: ShowcaseExampleMeta = {
  id: 'release-room',
  number: '03',
  title: 'Release room',
  eyebrow: 'Multi-page application',
  description: 'A complete generated product with route navigation, mounted visited pages, editable rollout state, evidence, and a final decision page.',
  capabilitySummary: 'Whole-page generation, local routing, preserved form state, and page-level art direction.',
  capabilities: ['Multi-page app', 'Preserved state', 'Local routing', 'Page templates'],
  protocols: ['A2UI', 'AG-UI', 'MCP Apps'],
  accent: '#d9472b',
};

export const brandCommerceMeta: ShowcaseExampleMeta = {
  id: 'brand-commerce',
  number: '04',
  title: 'Brand concierge',
  eyebrow: 'Design-system generation',
  description: 'A commerce concierge driven by an approved brand kit rather than generic component defaults, with a considered editorial hierarchy and guided selection.',
  capabilitySummary: 'Approved semantic tokens, distinct page art direction, guided comparison, and brand-safe actions.',
  capabilities: ['Brand kits', 'Semantic tokens', 'Editorial composition', 'Guided choice'],
  protocols: ['MCP Apps', 'A2UI'],
  accent: '#234f3f',
};

export const spatialLogisticsMeta: ShowcaseExampleMeta = {
  id: 'spatial-logistics',
  number: '05',
  title: 'Spatial logistics',
  eyebrow: 'Declarative 3D',
  description: 'A bounded spatial scene where semantic objects participate in the agent workflow without accepting arbitrary scripts, shaders, or unapproved assets.',
  capabilitySummary: 'Lazy WebGPU/WebGL rendering, semantic materials, labelled actions, and accessible fallback.',
  capabilities: ['Declarative 3D', 'Lazy renderer', 'Labelled actions', 'Safe asset boundary'],
  protocols: ['A2UI', 'MCP Apps'],
  accent: '#b8ff68',
};

export const streamingOperationsMeta: ShowcaseExampleMeta = {
  id: 'streaming-operations',
  number: '06',
  title: 'Streaming operations',
  eyebrow: 'Incremental UI',
  description: 'A surface assembled through targeted replace, patch, append, and remove operations instead of waiting for a complete response.',
  capabilitySummary: 'Component-addressed updates, progressive disclosure, stable surrounding state, and replayable transitions.',
  capabilities: ['Streaming updates', 'Patch + replace', 'Append + remove', 'Stable component state'],
  protocols: ['AG-UI', 'A2UI'],
  accent: '#2859d6',
};

export const SHOWCASE_MANIFEST: ShowcaseExampleMeta[] = [
  incidentCommandMeta,
  supportWorkflowMeta,
  releaseRoomMeta,
  brandCommerceMeta,
  spatialLogisticsMeta,
  streamingOperationsMeta,
];
