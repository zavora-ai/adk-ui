import type {
  Component,
  UiApplication,
  UiEvent,
  UiKitManifest,
  UiUpdate,
} from '@zavora-ai/adk-ui-react';

export interface ShowcaseExampleMeta {
  id: string;
  number: string;
  title: string;
  eyebrow: string;
  description: string;
  capabilitySummary: string;
  capabilities: string[];
  protocols: string[];
  accent: string;
}

interface ShowcaseExampleBase extends ShowcaseExampleMeta {
  kit: UiKitManifest;
}

export interface ApplicationShowcaseExample extends ShowcaseExampleBase {
  kind: 'application';
  application: UiApplication;
}

export interface StreamingShowcaseExample extends ShowcaseExampleBase {
  kind: 'streaming';
  initialComponent: Component;
  updates: UiUpdate[];
}

export type ShowcaseExample = ApplicationShowcaseExample | StreamingShowcaseExample;

export interface ShowcaseEventReceipt {
  label: string;
  detail: string;
  event?: UiEvent;
}
