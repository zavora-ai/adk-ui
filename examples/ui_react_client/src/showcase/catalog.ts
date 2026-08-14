import { validateUiApplication } from '@zavora-ai/adk-ui-react';
import { brandCommerceExample } from './examples/brand-commerce';
import { incidentCommandExample } from './examples/incident-command';
import { releaseRoomExample } from './examples/release-room';
import { spatialLogisticsExample } from './examples/spatial-logistics';
import { streamingOperationsExample } from './examples/streaming-operations';
import { supportWorkflowExample } from './examples/support-workflow';
import type { ShowcaseExample } from './types';

export const SHOWCASE_EXAMPLES: ShowcaseExample[] = [
  incidentCommandExample,
  supportWorkflowExample,
  releaseRoomExample,
  brandCommerceExample,
  spatialLogisticsExample,
  streamingOperationsExample,
];

const showcaseIds = new Set<string>();
for (const example of SHOWCASE_EXAMPLES) {
  if (showcaseIds.has(example.id)) {
    throw new Error(`Duplicate showcase id: ${example.id}`);
  }
  showcaseIds.add(example.id);

  if (example.kind === 'application') {
    const errors = validateUiApplication(example.application);
    if (errors.length > 0) {
      throw new Error(`Invalid showcase application ${example.id}: ${errors.join(', ')}`);
    }
  }
}

export function findShowcaseExample(id: string | null): ShowcaseExample | null {
  if (!id) return null;
  return SHOWCASE_EXAMPLES.find((example) => example.id === id) ?? null;
}
