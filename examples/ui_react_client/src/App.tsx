import { useEffect, useMemo, useRef, useState } from 'react';
import { Renderer as UiRenderer } from './adk-ui-renderer/Renderer';
import { convertA2UIComponent } from './adk-ui-renderer/a2ui-converter';
import { uiEventToMessage, type Component, type UiEvent } from './adk-ui-renderer/types';
import {
  MCP_APPS_PROTOCOL_VERSION,
  buildMcpAppsInitializeRequest,
  buildOutboundEvent,
  extractProtocolSurface,
  type ProtocolSurfaceSnapshot as SharedProtocolSurfaceSnapshot,
} from '../../../packages/adk-ui-react/src/index.ts';
import './App.css';

type UiProtocol = 'adk_ui' | 'a2ui' | 'ag_ui' | 'mcp_apps' | 'awp';

interface SurfaceSnapshot {
  protocol?: SharedProtocolSurfaceSnapshot['protocol'];
  source?: SharedProtocolSurfaceSnapshot['source'];
  surfaceId: string;
  components: Component[];
  dataModel: Record<string, unknown>;
  bridge?: SharedProtocolSurfaceSnapshot['bridge'];
  /** AWP-rendered HTML for iframe display */
  html?: string;
}

interface StreamLogEvent {
  id: number;
  at: string;
  protocol: UiProtocol;
  kind: string;
  preview: string;
  raw: unknown;
}

interface AgUiActivityMessage {
  messageId: string;
  activityType: string;
  content: Record<string, unknown>;
  replace: boolean;
}

type JsonPatchOperation = {
  op?: unknown;
  path?: unknown;
  from?: unknown;
  value?: unknown;
};

interface ExampleTarget {
  id: string;
  name: string;
  description: string;
  port: number;
  prompts: string[];
}

interface ProtocolCapability {
  protocol: string;
  versions: string[];
  implementationTier?: string;
  specTrack?: string;
  summary?: string;
  features: string[];
  limitations?: string[];
  deprecation?: {
    stage: string;
    announcedOn: string;
    sunsetTargetOn?: string;
    replacementProtocols: string[];
    note?: string;
  };
}

interface ActionRequestBridge {
  outbound: Record<string, unknown>;
  requestBody: Record<string, unknown>;
  textMessage: string;
  preview: string;
  bridgeRequest?: {
    endpoint: string;
    body: Record<string, unknown>;
    kind: string;
    preview: string;
    skipModelRun?: boolean;
  };
}

interface McpAppsBridgeSessionState {
  sessionId?: string;
}

type TableColumnDef = {
  header: string;
  accessor_key: string;
  sortable?: boolean;
};

const EXAMPLES: ExampleTarget[] = [
  {
    id: 'ui_demo',
    name: 'UI Demo',
    description: 'General purpose multi-surface demo',
    port: 8080,
    prompts: [
      'Create a dashboard with three KPI cards and a trend chart.',
      'Design an onboarding form with progress, validation hints, and submit actions.',
      'Build an operations command center with alerts, table, and a confirmation modal.',
    ],
  },
  {
    id: 'ui_working_support',
    name: 'Support Intake',
    description: 'Ticket intake and triage workflows',
    port: 8080,
    prompts: [
      'Build a support intake flow with severity selector, timeline, and submit button.',
      'Show an incident response board with ownership, status, and next steps.',
      'Create a postmortem form with root-cause sections and action items.',
    ],
  },
  {
    id: 'ui_working_appointment',
    name: 'Appointments',
    description: 'Scheduling and availability workflows',
    port: 8080,
    prompts: [
      'Render an appointment booking experience with service cards and time slots.',
      'Create a multi-step reschedule workflow with confirmation state.',
      'Show a clinician schedule table with status badges and reminders.',
    ],
  },
  {
    id: 'ui_working_events',
    name: 'Events',
    description: 'Registration and agenda workflows',
    port: 8080,
    prompts: [
      'Build an event registration UI with ticket options and attendee details.',
      'Render an agenda timeline with speaker cards and room map summary.',
      'Design a networking dashboard with RSVP stats and waitlist table.',
    ],
  },
  {
    id: 'ui_working_facilities',
    name: 'Facilities',
    description: 'Maintenance requests and escalation',
    port: 8080,
    prompts: [
      'Create a facilities issue form with priority, location, and SLA warning.',
      'Show a maintenance dispatch board with progress, owners, and alerts.',
      'Render a work-order detail view with checklist and completion modal.',
    ],
  },
  {
    id: 'ui_working_inventory',
    name: 'Inventory',
    description: 'Stock monitoring and restock workflows',
    port: 8080,
    prompts: [
      'Build an inventory monitor with low-stock alerts and reorder actions.',
      'Create a replenishment form with quantity controls and approval summary.',
      'Render supplier performance cards plus a lead-time trend chart.',
    ],
  },
];

const PROTOCOLS: Array<{ id: UiProtocol; label: string; hint: string }> = [
  {
    id: 'adk_ui',
    label: 'Legacy ADK UI',
    hint: 'Backward-compatible internal profile (deprecated).',
  },
  {
    id: 'a2ui',
    label: 'A2UI',
    hint: 'Core surface subset with draft v0.9 alignment.',
  },
  {
    id: 'ag_ui',
    label: 'AG-UI',
    hint: 'Hybrid subset with protocol-native runtime transport, AG-UI run input, and stable text/tool event parsing.',
  },
  {
    id: 'mcp_apps',
    label: 'MCP Apps',
    hint: 'Compatibility subset with framework bridge endpoints, bridge-aware structured content, ui:// resources, and inline HTML fallback.',
  },
  {
    id: 'awp',
    label: 'AWP',
    hint: 'Agentic Web Protocol — dual-user rendering with HTML output, capability manifest export, and bandwidth-adaptive mode.',
  },
];

const MAX_EVENT_LOG = 120;
const DEFAULT_USER_ID = 'user1';

function nowLabel(): string {
  return new Date().toLocaleTimeString('en-US', { hour12: false });
}

function makeSessionId(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  // Fallback for environments without crypto.randomUUID — uses crypto.getRandomValues
  // for better randomness than Math.random().
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    const bytes = new Uint8Array(16);
    crypto.getRandomValues(bytes);
    return `session-${Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')}`;
  }
  return `session-${Date.now()}-${Math.floor(Math.random() * 1_000_000)}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function pickField(record: Record<string, unknown>, ...keys: string[]): unknown {
  for (const key of keys) {
    if (key in record) {
      return record[key];
    }
  }
  return undefined;
}

function parseMaybeJson(value: unknown): unknown {
  if (typeof value !== 'string') {
    return value;
  }
  const trimmed = value.trim();
  if (!trimmed.startsWith('{') && !trimmed.startsWith('[')) {
    return value;
  }
  try {
    return JSON.parse(trimmed);
  } catch {
    return value;
  }
}

function normalizeProtocol(value: unknown): UiProtocol | null {
  if (typeof value !== 'string') {
    return null;
  }
  const normalized = value.trim().toLowerCase();
  if (normalized === 'adk_ui' || normalized === 'adk-ui') return 'adk_ui';
  if (normalized === 'a2ui') return 'a2ui';
  if (normalized === 'ag_ui' || normalized === 'ag-ui') return 'ag_ui';
  if (normalized === 'mcp_apps' || normalized === 'mcp-apps') return 'mcp_apps';
  if (normalized === 'awp') return 'awp';
  return null;
}

function takeCompleteSseEvents(buffer: string): { events: string[]; rest: string } {
  let rest = buffer.replace(/\r\n/g, '\n');
  const events: string[] = [];

  while (true) {
    const boundary = rest.indexOf('\n\n');
    if (boundary < 0) {
      break;
    }
    events.push(rest.slice(0, boundary));
    rest = rest.slice(boundary + 2);
  }

  return { events, rest };
}

function extractSseEventPayload(rawEvent: string): string | null {
  const dataLines: string[] = [];

  for (const rawLine of rawEvent.split('\n')) {
    const line = rawLine.trimEnd();
    if (!line || line.startsWith(':') || !line.startsWith('data:')) {
      continue;
    }
    dataLines.push(line.slice(5).trimStart());
  }

  if (dataLines.length === 0) {
    return null;
  }

  const payload = dataLines.join('\n').trim();
  return payload.length > 0 ? payload : null;
}

function extractTextPartsFromContent(content: unknown): string[] {
  if (!Array.isArray(content)) {
    return [];
  }

  const snippets: string[] = [];
  for (const entry of content) {
    if (!isRecord(entry)) {
      continue;
    }

    const text = pickField(entry, 'text');
    if (typeof text === 'string' && text.trim().length > 0) {
      snippets.push(text.trim());
    }
  }

  return snippets;
}

function extractOutboundTextMessage(
  outbound: Record<string, unknown>,
  fallback: string,
): string {
  const protocol = normalizeProtocol(pickField(outbound, 'protocol'));
  if (protocol === 'ag_ui') {
    const input = pickField(outbound, 'input');
    if (isRecord(input)) {
      const messages = pickField(input, 'messages');
      if (Array.isArray(messages)) {
        const snippets: string[] = [];
        for (const entry of messages) {
          if (!isRecord(entry)) {
            continue;
          }
          const content = pickField(entry, 'content');
          if (typeof content === 'string' && content.trim().length > 0) {
            snippets.push(content.trim());
          }
        }
        if (snippets.length > 0) {
          return snippets.join('\n\n');
        }
      }
    }
  }

  if (protocol === 'mcp_apps') {
    const params = pickField(outbound, 'params');
    if (isRecord(params)) {
      const content = pickField(params, 'content');
      const snippets = extractTextPartsFromContent(content);
      if (snippets.length > 0) {
        return snippets.join('\n\n');
      }
    }
  }

  return fallback;
}

function describeOutboundEvent(protocol: UiProtocol, outbound: Record<string, unknown>): string {
  if (protocol === 'mcp_apps') {
    const method = pickField(outbound, 'method');
    if (typeof method === 'string' && method.trim().length > 0) {
      return method;
    }
  }

  if (protocol === 'ag_ui') {
    return 'ag_ui run input';
  }

  return `${protocol} event`;
}

function buildActionRequestBridge(
  protocol: UiProtocol,
  action: UiEvent,
  context: {
    appName: string;
    userId: string;
    sessionId: string;
    surfaceId: string;
    threadId?: string;
    parentRunId?: string;
  },
): ActionRequestBridge {
  const timestamp = Date.now();
  const outbound = buildOutboundEvent(protocol, action, {
    surfaceId: context.surfaceId,
    threadId: context.threadId ?? `thread-${context.sessionId}`,
    runId: `run-${context.sessionId}-${timestamp}`,
    parentRunId: context.parentRunId,
    messageId: `msg-${context.sessionId}-${timestamp}`,
  });
  const fallbackMessage = uiEventToMessage(action);
  const textMessage = extractOutboundTextMessage(outbound, fallbackMessage);
  const preview = describeOutboundEvent(protocol, outbound);
  const requestBody: Record<string, unknown> = {};
  let bridgeRequest: ActionRequestBridge['bridgeRequest'];

  if (protocol === 'ag_ui') {
    requestBody.uiTransport = 'protocol_native';
    requestBody.input = buildAgUiRuntimeInput(textMessage, {
      sessionId: context.sessionId,
      surfaceId: context.surfaceId,
      threadId: context.threadId,
      parentRunId: context.parentRunId,
      runId: `run-${context.sessionId}-${timestamp}`,
      messageId: `msg-${context.sessionId}-${timestamp}`,
      forwardedProps: {
        uiEvent: action,
      },
    });
  } else if (protocol === 'mcp_apps') {
    const method = pickField(outbound, 'method');
    const params = pickField(outbound, 'params');

    if (typeof method === 'string' && isRecord(params)) {
      const endpoint = resolveMcpAppsBridgeEndpoint(method);
      if (endpoint) {
        bridgeRequest = {
          endpoint,
          kind:
            method === 'ui/update-model-context'
              ? 'mcp_apps_model_context'
              : 'mcp_apps_message',
          preview: method,
          skipModelRun: method === 'ui/update-model-context',
          body: {
            appName: context.appName,
            userId: context.userId,
            sessionId: context.sessionId,
            ...params,
          },
        };
      }
    }
  } else {
    const protocolEvent = pickField(outbound, 'event');
    if (protocolEvent !== undefined) {
      requestBody.protocolEvent = protocolEvent;
    }
  }

  return {
    outbound,
    requestBody,
    textMessage,
    preview,
    bridgeRequest,
  };
}

function buildMcpAppsHandshakePayload(): {
  initializeRequest: Record<string, unknown>;
} {
  return {
    initializeRequest: buildMcpAppsInitializeRequest({
      protocolVersion: MCP_APPS_PROTOCOL_VERSION,
      appInfo: {
        name: 'adk-ui-react-example',
        version: '0.4.0',
        title: 'ADK UI Example View',
        description: 'Example MCP Apps-compatible view host for adk-ui.',
      },
      appCapabilities: {
        availableDisplayModes: ['inline'],
        tools: {
          listChanged: false,
        },
      },
    }) as unknown as Record<string, unknown>,
  };
}

function buildAgUiRuntimeInput(
  message: string,
  context: {
    sessionId: string;
    surfaceId: string;
    threadId?: string;
    parentRunId?: string;
    runId?: string;
    messageId?: string;
    forwardedProps?: Record<string, unknown>;
    activityMessages?: Array<Record<string, unknown>>;
  },
): Record<string, unknown> {
  const timestamp = Date.now();

  return {
    threadId: context.threadId ?? `thread-${context.sessionId}`,
    runId: context.runId ?? `run-${context.sessionId}-${timestamp}`,
    parentRunId: context.parentRunId,
    messages: [
      {
        id: context.messageId ?? `msg-${context.sessionId}-${timestamp}`,
        role: 'user',
        name: 'adk-ui',
        content: [
          {
            type: 'text',
            text: message,
          },
        ],
      },
      ...(context.activityMessages ?? []),
    ],
    tools: [],
    context: [],
    forwardedProps: {
      source: 'adk-ui-react-example',
      surfaceId: context.surfaceId,
      ...(context.forwardedProps ?? {}),
    },
  };
}

function buildPromptContextActivityMessages(
  prompt: string,
  context: {
    sessionId: string;
    exampleId: string;
    retryAttempt: number;
    protocol: UiProtocol;
  },
): Array<Record<string, unknown>> {
  const timestamp = Date.now();
  const messageId = `activity-${context.sessionId}-${timestamp}`;
  return [
    {
      id: messageId,
      role: 'activity',
      activityType: 'PROMPT_CONTEXT',
      replace: true,
      content: {
        label: context.retryAttempt > 0 ? 'Retry prompt queued' : 'Prompt queued',
        prompt,
        exampleId: context.exampleId,
        retryAttempt: context.retryAttempt,
        status: context.retryAttempt > 0 ? 'queued_retry' : 'queued',
      },
    },
    {
      id: messageId,
      role: 'activity',
      activityType: 'PROMPT_CONTEXT',
      replace: false,
      patch: [
        { op: 'replace', path: '/status', value: 'streaming' },
        { op: 'add', path: '/transport', value: context.protocol === 'ag_ui' ? 'protocol_native' : context.protocol },
      ],
    },
  ];
}

function resolveMcpAppsBridgeEndpoint(method: string): string | null {
  if (method === 'ui/initialize') return '/api/ui/initialize';
  if (method === 'ui/message') return '/api/ui/message';
  if (method === 'ui/update-model-context') return '/api/ui/update-model-context';
  return null;
}

function buildMcpAppsInitializeBody(
  appName: string,
  userId: string,
  sessionId: string,
): Record<string, unknown> {
  const handshake = buildMcpAppsHandshakePayload();
  const params = isRecord(handshake.initializeRequest.params)
    ? handshake.initializeRequest.params
    : {};

  return {
    appName,
    userId,
    sessionId,
    ...params,
  };
}

function formatCapabilityToken(value: string): string {
  return value
    .split('_')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ');
}

function protocolInstruction(protocol: UiProtocol): string {
  if (protocol === 'mcp_apps') {
    return [
      'Use adk-ui tools and explicitly set `protocol` to `mcp_apps`.',
      'Assume the host supports MCP Apps initialize/message/model-context bridge flows.',
      'Include `mcp_apps.domain` as `https://example.com` when you call render tools.',
      'Prefer rich components (cards, tables, charts, alerts, forms) instead of plain text-only layouts.',
      'Prefer `render_layout`, `render_table`, `render_chart`, and `render_form` when they fit the request.',
      'Call at least one `render_*` tool before any plain text response.',
      'If rendering fails, call `render_alert` with an error message.',
      'Return rich UI with forms, charts, and actionable controls.',
    ].join(' ');
  }
  if (protocol === 'awp') {
    return [
      'Use adk-ui tools and explicitly set `protocol` to `awp`.',
      'AWP (Agentic Web Protocol) produces both a structured component tree and rendered HTML in a single response.',
      'The runtime will generate embeddable HTML from your components automatically.',
      'Prefer rich components (cards, tables, charts, alerts, forms) instead of plain text-only layouts.',
      'Prefer `render_layout`, `render_table`, `render_chart`, and `render_form` when they fit the request.',
      'Call at least one `render_*` tool before any plain text response.',
      'If rendering fails, call `render_alert` with an error message.',
      'Return rich UI with layouts, data visuals, and clear actions.',
    ].join(' ');
  }
  return [
    `Use adk-ui tools and explicitly set \`protocol\` to \`${protocol}\`.`,
    protocol === 'ag_ui'
      ? 'Assume the runtime supports protocol-native AG-UI transport with stable lifecycle, text, and tool events.'
      : 'Assume the runtime preserves protocol-aware payloads alongside the base ADK stream.',
    'Prefer rich components (cards, tables, charts, alerts, forms) instead of plain text-only layouts.',
    'Prefer `render_layout`, `render_table`, `render_chart`, and `render_form` when they fit the request.',
    'Call at least one `render_*` tool before any plain text response.',
    'If rendering fails, call `render_alert` with an error message.',
    'Return rich UI with layouts, data visuals, and clear actions.',
  ].join(' ');
}

function retryInstruction(
  protocol: UiProtocol,
  mode: 'no_surface' | 'quality' = 'no_surface',
): string {
  if (mode === 'quality') {
    if (protocol === 'mcp_apps') {
      return [
        'Retry mode: previous attempt produced a low-fidelity UI (mostly text/buttons).',
        'Now produce a richer surface using render tools: include at least one table, one card, and one alert.',
        'Use render_layout for composition and avoid plain text-only timelines/lists.',
        'Do not emit plain text before tool calls.',
      ].join(' ');
    }

    return [
      'Retry mode: previous attempt produced a low-fidelity UI (mostly text/buttons).',
      `Now produce a richer \`${protocol}\` surface using render tools: include at least one table, one card, and one alert.`,
      'Use render_layout for composition and avoid plain text-only timelines/lists.',
      'Do not emit plain text before tool calls.',
    ].join(' ');
  }

  if (protocol === 'mcp_apps') {
    return [
      'Retry mode: previous attempt produced no renderable UI.',
      'Now emit exactly one valid `render_screen` tool call immediately with `protocol: "mcp_apps"` and a `root` component.',
      'Ensure the surface includes structured UI (table/card/alert), not only text and buttons.',
      'Do not emit plain text before the tool call.',
    ].join(' ');
  }
  return [
    'Retry mode: previous attempt produced no renderable UI.',
    `Now emit exactly one valid \`render_screen\` tool call immediately with \`protocol: "${protocol}"\` and a \`root\` component.`,
    'Ensure the surface includes structured UI (table/card/alert), not only text and buttons.',
    'Do not emit plain text before the tool call.',
  ].join(' ');
}

function extractTextValue(value: unknown): string {
  if (typeof value === 'string') {
    return value;
  }
  if (!isRecord(value)) {
    return '';
  }
  const literal = pickField(value, 'literalString', 'literal_string');
  if (typeof literal === 'string') {
    return literal;
  }
  const dynamic = pickField(value, 'dynamicString', 'dynamic_string');
  if (typeof dynamic === 'string') {
    return dynamic;
  }
  const text = pickField(value, 'text');
  return typeof text === 'string' ? text : '';
}

function extractLegacyUiResponseComponents(dataModel: unknown): Component[] | null {
  if (!isRecord(dataModel)) {
    return null;
  }

  const legacy = pickField(dataModel, 'adk_ui_response', 'adkUiResponse');
  if (!isRecord(legacy)) {
    return null;
  }

  const rawComponents = pickField(legacy, 'components');
  if (!Array.isArray(rawComponents)) {
    return null;
  }

  const components = rawComponents
    .filter((entry): entry is Record<string, unknown> => isRecord(entry))
    .filter((entry) => typeof pickField(entry, 'type') === 'string')
    .map((entry) => entry as unknown as Component);

  return components.length > 0 ? components : null;
}

function parseTableColumnDef(entry: Record<string, unknown>): TableColumnDef | null {
  const componentField = pickField(entry, 'component');
  let tableColumnNode: Record<string, unknown> | null = null;

  if (isRecord(componentField)) {
    const candidate = pickField(componentField, 'TableColumn', 'tableColumn', 'table_column', 'column');
    if (isRecord(candidate)) {
      tableColumnNode = candidate;
    } else {
      const maybeAccessor = pickField(componentField, 'accessorKey', 'accessor_key', 'accessor');
      if (typeof maybeAccessor === 'string') {
        tableColumnNode = componentField;
      }
    }
  } else if (typeof componentField === 'string') {
    const normalized = componentField.toLowerCase();
    if (normalized === 'tablecolumn' || normalized === 'table_column' || normalized === 'table-column') {
      tableColumnNode = entry;
    }
  }

  if (!tableColumnNode) {
    const direct = pickField(entry, 'TableColumn', 'tableColumn', 'table_column');
    if (isRecord(direct)) {
      tableColumnNode = direct;
    }
  }

  if (!tableColumnNode) {
    return null;
  }

  const accessor = pickField(tableColumnNode, 'accessorKey', 'accessor_key', 'accessor');
  if (typeof accessor !== 'string' || accessor.trim().length === 0) {
    return null;
  }

  const headerRaw = pickField(tableColumnNode, 'header', 'title', 'label');
  const header = extractTextValue(headerRaw) || (typeof headerRaw === 'string' ? headerRaw : '') || humanizeId(accessor);
  const sortableRaw = pickField(tableColumnNode, 'sortable');
  const sortable = typeof sortableRaw === 'boolean' ? sortableRaw : undefined;

  return {
    header,
    accessor_key: accessor,
    sortable,
  };
}

function extractTableColumnDefs(
  rawComponents: unknown[],
): Map<string, TableColumnDef> {
  const refs = new Map<string, TableColumnDef>();

  for (const entry of rawComponents) {
    if (!isRecord(entry)) {
      continue;
    }

    const columnId = typeof pickField(entry, 'id') === 'string' ? (pickField(entry, 'id') as string) : null;
    if (!columnId) {
      continue;
    }

    const def = parseTableColumnDef(entry);
    if (!def) {
      continue;
    }

    refs.set(columnId, def);
  }

  return refs;
}

function findTableColumnDef(rawComponents: unknown[], columnRef: string): TableColumnDef | null {
  for (const entry of rawComponents) {
    if (!isRecord(entry)) {
      continue;
    }
    const id = pickField(entry, 'id');
    if (id !== columnRef) {
      continue;
    }
    const parsed = parseTableColumnDef(entry);
    if (parsed) {
      return parsed;
    }
  }
  return null;
}

function inferAccessorFromRef(columnRef: string, sampleKeys: string[], columnIndex: number): string | null {
  if (sampleKeys.length === 0) {
    return null;
  }

  const lowered = columnRef.toLowerCase();
  const normalized = lowered.replace(/^col[-_]/, '').replace(/^column[-_]/, '');
  const compact = normalized.replace(/[-_]/g, '');

  const direct = sampleKeys.find((key) => key.toLowerCase() === lowered);
  if (direct) {
    return direct;
  }

  const normalizedMatch = sampleKeys.find((key) => key.toLowerCase() === normalized);
  if (normalizedMatch) {
    return normalizedMatch;
  }

  const compactMatch = sampleKeys.find((key) => key.toLowerCase().replace(/[-_]/g, '') === compact);
  if (compactMatch) {
    return compactMatch;
  }

  if (columnIndex >= 0 && columnIndex < sampleKeys.length) {
    return sampleKeys[columnIndex];
  }

  return null;
}

function inferTableColumnDef(
  columnRef: string,
  sampleRow: Record<string, unknown> | null,
  columnIndex: number,
): TableColumnDef | null {
  if (!sampleRow) {
    return null;
  }
  const keys = Object.keys(sampleRow);
  const accessor = inferAccessorFromRef(columnRef, keys, columnIndex);
  if (!accessor) {
    return null;
  }
  return {
    header: humanizeId(accessor),
    accessor_key: accessor,
  };
}

function normalizeRequestError(error: unknown, baseUrl: string, endpoint: string, port: number): string {
  const fallback = `Request to ${baseUrl}${endpoint} failed.`;
  if (!(error instanceof Error)) {
    return fallback;
  }

  const message = error.message || fallback;
  const networkPattern =
    /failed to fetch|load failed|networkerror|fetch failed|could not connect|connection refused|request blocked/i;
  if (networkPattern.test(message)) {
    return `Cannot reach ${baseUrl}${endpoint}. Start the example server on port ${port} and retry.`;
  }

  return message;
}

function extractButtonChildRefs(rawComponents: unknown[]): Map<string, string> {
  const refs = new Map<string, string>();

  for (const entry of rawComponents) {
    if (!isRecord(entry)) {
      continue;
    }

    const buttonId = typeof pickField(entry, 'id') === 'string' ? (pickField(entry, 'id') as string) : null;
    if (!buttonId) {
      continue;
    }

    const componentField = pickField(entry, 'component');
    if (isRecord(componentField)) {
      const buttonNode = pickField(componentField, 'Button');
      if (isRecord(buttonNode)) {
        const childRef = pickField(buttonNode, 'child');
        if (typeof childRef === 'string' && childRef.trim().length > 0) {
          refs.set(buttonId, childRef);
        }
      }
      continue;
    }

    if (typeof componentField === 'string' && componentField === 'Button') {
      const childRef = pickField(entry, 'child');
      if (typeof childRef === 'string' && childRef.trim().length > 0) {
        refs.set(buttonId, childRef);
      }
    }
  }

  return refs;
}

function humanizeId(id: string): string {
  const text = id
    .replace(/[_-]+/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  if (text.length === 0) {
    return 'Submit';
  }
  return text.replace(/\b\w/g, (char) => char.toUpperCase());
}

function convertRawComponent(raw: unknown, fallbackId: string): { id?: string; component: Component } | null {
  if (!isRecord(raw)) {
    return null;
  }

  if (typeof raw.type === 'string') {
    const component = raw as unknown as Component;
    return {
      id: typeof raw.id === 'string' ? raw.id : undefined,
      component,
    };
  }

  const componentField = pickField(raw, 'component');

  let componentObject: Record<string, unknown> | null = null;
  if (isRecord(componentField)) {
    componentObject = componentField;
  } else if (typeof componentField === 'string') {
    // Support adk-ui flat payload shape: { component: "Text", text: "...", ... }
    const inlineProps: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(raw)) {
      if (key === 'id' || key === 'component') {
        continue;
      }
      inlineProps[key] = value;
    }
    componentObject = { [componentField]: inlineProps };
  }

  if (!componentObject) {
    return null;
  }

  const sourceId =
    typeof pickField(raw, 'id') === 'string' ? (pickField(raw, 'id') as string) : fallbackId;
  const converted = convertA2UIComponent({
    id: sourceId,
    component: componentObject,
  });

  if (!converted) {
    return null;
  }

  return {
    id: sourceId,
    component: converted,
  };
}

function resolveNestedComponent(
  component: Component,
  byId: Map<string, Component>,
  path: Set<string>,
): Component {
  const clone = { ...(component as Record<string, unknown>) } as Record<string, unknown>;
  const currentId = typeof clone.id === 'string' ? clone.id : undefined;

  if (currentId) {
    if (path.has(currentId)) {
      return component;
    }
    path.add(currentId);
  }

  if (Array.isArray(clone.children)) {
    clone.children = clone.children
      .map((entry) => {
        if (typeof entry === 'string') {
          return byId.get(entry);
        }
        return entry;
      })
      .filter((entry): entry is Component => Boolean(entry))
      .map((entry) => resolveNestedComponent(entry, byId, new Set(path)));
  }

  if (Array.isArray(clone.content)) {
    clone.content = clone.content
      .map((entry) => {
        if (typeof entry === 'string') {
          return byId.get(entry);
        }
        return entry;
      })
      .filter((entry): entry is Component => Boolean(entry) && isRecord(entry))
      .map((entry) => resolveNestedComponent(entry, byId, new Set(path)));
  }

  if (Array.isArray(clone.footer)) {
    clone.footer = clone.footer
      .map((entry) => {
        if (typeof entry === 'string') {
          return byId.get(entry);
        }
        return entry;
      })
      .filter((entry): entry is Component => Boolean(entry) && isRecord(entry))
      .map((entry) => resolveNestedComponent(entry, byId, new Set(path)));
  }

  if (Array.isArray(clone.tabs)) {
    clone.tabs = clone.tabs
      .filter((entry) => isRecord(entry) && Array.isArray(entry.content))
      .map((tab) => ({
        ...tab,
        content: (tab.content as unknown[])
          .map((entry) => {
            if (typeof entry === 'string') {
              return byId.get(entry);
            }
            return entry;
          })
          .filter((entry): entry is Component => Boolean(entry) && isRecord(entry))
          .map((entry) => resolveNestedComponent(entry, byId, new Set(path))),
      }));
  }

  return clone as Component;
}

function buildRenderableComponents(rawComponents: unknown[]): Component[] {
  const byId = new Map<string, Component>();
  const withoutId: Component[] = [];
  const buttonChildRefs = extractButtonChildRefs(rawComponents);
  const tableColumnDefs = extractTableColumnDefs(rawComponents);

  rawComponents.forEach((entry, index) => {
    const converted = convertRawComponent(entry, `component-${index + 1}`);
    if (!converted) {
      return;
    }
    if (converted.id) {
      byId.set(converted.id, converted.component);
    } else {
      withoutId.push(converted.component);
    }
  });

  // A2UI buttons may reference a child Text node by id. Hydrate label from that node.
  for (const [buttonId, childId] of buttonChildRefs) {
    const buttonNode = byId.get(buttonId);
    if (!buttonNode || buttonNode.type !== 'button') {
      continue;
    }
    const childNode = byId.get(childId);
    const textLabel =
      childNode && childNode.type === 'text' && childNode.content.trim().length > 0
        ? childNode.content
        : null;

    const currentLabel = buttonNode.label?.trim() ?? '';
    if (currentLabel.length === 0 || currentLabel === childId) {
      byId.set(buttonId, {
        ...buttonNode,
        label: textLabel ?? humanizeId(childId),
      });
    }
  }

  // A2UI tables can declare columns as ids and define TableColumn components separately.
  for (const [componentId, component] of byId.entries()) {
    if (component.type !== 'table') {
      continue;
    }

    const rawColumns = (component as unknown as { columns?: unknown[] }).columns;
    if (!Array.isArray(rawColumns)) {
      continue;
    }
    const rawData = (component as unknown as { data?: unknown[] }).data;
    const sampleRow =
      Array.isArray(rawData) && rawData.length > 0 && isRecord(rawData[0])
        ? (rawData[0] as Record<string, unknown>)
        : null;

    const hydrated = rawColumns
      .map((column, index) => {
        if (typeof column === 'string') {
          const fromMap = tableColumnDefs.get(column);
          if (fromMap) {
            return fromMap;
          }
          const fromRaw = findTableColumnDef(rawComponents, column);
          if (fromRaw) {
            return fromRaw;
          }
          return inferTableColumnDef(column, sampleRow, index);
        }
        if (isRecord(column)) {
          const accessor = pickField(column, 'accessor_key', 'accessorKey');
          if (typeof accessor !== 'string' || accessor.trim().length === 0) {
            return null;
          }
          const header = extractTextValue(pickField(column, 'header')) || humanizeId(accessor);
          const sortableRaw = pickField(column, 'sortable');
          const sortable = typeof sortableRaw === 'boolean' ? sortableRaw : undefined;
          return {
            header,
            accessor_key: accessor,
            sortable,
          };
        }
        return null;
      })
      .filter(
        (column): column is { header: string; accessor_key: string; sortable?: boolean } =>
          Boolean(column),
      );

    if (hydrated.length > 0) {
      byId.set(componentId, {
        ...component,
        columns: hydrated,
      });
    }
  }

  const referenced = new Set<string>();
  byId.forEach((component) => {
    const node = component as Record<string, unknown>;
    const maybeChildren = node.children;
    const maybeContent = node.content;
    const maybeFooter = node.footer;
    const maybeTabs = node.tabs;

    const collectRefs = (entries: unknown) => {
      if (!Array.isArray(entries)) {
        return;
      }
      for (const entry of entries) {
        if (typeof entry === 'string') {
          referenced.add(entry);
        }
      }
    };

    collectRefs(maybeChildren);
    collectRefs(maybeContent);
    collectRefs(maybeFooter);

    if (Array.isArray(maybeTabs)) {
      for (const tab of maybeTabs) {
        if (!isRecord(tab)) {
          continue;
        }
        collectRefs(tab.content);
      }
    }
  });

  if (byId.has('root')) {
    return [resolveNestedComponent(byId.get('root') as Component, byId, new Set())];
  }

  const rootCandidates = Array.from(byId.entries())
    .filter(([id]) => !referenced.has(id))
    .map(([, component]) => resolveNestedComponent(component, byId, new Set()));

  if (rootCandidates.length > 0) {
    return [...rootCandidates, ...withoutId];
  }

  const fallback = Array.from(byId.values()).map((component) =>
    resolveNestedComponent(component, byId, new Set()),
  );

  return [...fallback, ...withoutId];
}

function extractSurfaceFromProtocolSnapshot(
  surface: SharedProtocolSurfaceSnapshot,
): SurfaceSnapshot | null {
  const dataModel = isRecord(surface.dataModel) ? surface.dataModel : {};
  const legacyComponents = extractLegacyUiResponseComponents(dataModel);
  const components = legacyComponents ?? buildRenderableComponents(surface.components);
  if (components.length === 0) {
    return null;
  }

  return {
    protocol: surface.protocol,
    source: surface.source,
    surfaceId: surface.surfaceId || 'main',
    components,
    dataModel,
    bridge: surface.bridge ?? null,
  };
}

function extractSurfaceFromToolResponse(response: unknown): SurfaceSnapshot | null {
  const surface = extractProtocolSurface(response);
  if (!surface) {
    return null;
  }

  const snapshot = extractSurfaceFromProtocolSnapshot(surface);
  if (!snapshot) {
    return null;
  }

  // Extract AWP HTML from the payload for iframe rendering
  if (snapshot.protocol === 'awp' && isRecord(response)) {
    const payload = isRecord(response) ? (pickField(response as Record<string, unknown>, 'payload') as Record<string, unknown> | undefined) : undefined;
    const html = payload && typeof payload === 'object' ? (payload as Record<string, unknown>).html : undefined;
    if (typeof html === 'string' && html.length > 0) {
      snapshot.html = html;
    }
  }

  return snapshot;
}

function normalizeAlertVariant(value: unknown): 'info' | 'success' | 'warning' | 'error' {
  const normalized = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (normalized === 'success') return 'success';
  if (normalized === 'warning' || normalized === 'warn') return 'warning';
  if (normalized === 'error' || normalized === 'danger') return 'error';
  return 'info';
}

function normalizeButtonVariant(
  value: unknown,
): 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline' {
  const normalized = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (normalized === 'secondary') return 'secondary';
  if (normalized === 'danger' || normalized === 'error' || normalized === 'destructive') {
    return 'danger';
  }
  if (normalized === 'ghost') return 'ghost';
  if (normalized === 'outline') return 'outline';
  return 'primary';
}

function normalizeBadgeVariant(
  value: unknown,
): 'default' | 'info' | 'success' | 'warning' | 'error' | 'secondary' | 'outline' {
  const normalized = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (normalized === 'operational' || normalized === 'ok' || normalized === 'success') {
    return 'success';
  }
  if (normalized === 'degraded' || normalized === 'warning' || normalized === 'warn') {
    return 'warning';
  }
  if (normalized === 'down' || normalized === 'error' || normalized === 'outage') {
    return 'error';
  }
  if (normalized === 'info') {
    return 'info';
  }
  return 'secondary';
}

function normalizeChartKind(value: unknown): 'bar' | 'line' | 'area' | 'pie' {
  const normalized = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (normalized === 'line') return 'line';
  if (normalized === 'area') return 'area';
  if (normalized === 'pie') return 'pie';
  return 'bar';
}

function normalizeModalSize(value: unknown): 'small' | 'medium' | 'large' | 'full' {
  const normalized = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (normalized === 'small') return 'small';
  if (normalized === 'large') return 'large';
  if (normalized === 'full') return 'full';
  return 'medium';
}

function stringifyDisplayValue(value: unknown): string {
  if (typeof value === 'string') {
    return value;
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value);
  }
  if (value === null || value === undefined) {
    return '';
  }
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function normalizeSurfaceIdFromResponse(response: Record<string, unknown>): string {
  const surfaceId = pickField(response, 'surface_id', 'surfaceId');
  return typeof surfaceId === 'string' && surfaceId.trim().length > 0 ? surfaceId : 'main';
}

function normalizeTableRows(value: unknown): Record<string, unknown>[] {
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((entry): entry is Record<string, unknown> => isRecord(entry));
}

function normalizeTableColumns(
  value: unknown,
  rows: Record<string, unknown>[],
): Array<{ header: string; accessor_key: string }> {
  if (Array.isArray(value)) {
    const normalized = value
      .filter((entry): entry is Record<string, unknown> => isRecord(entry))
      .map((entry) => {
        const accessor = pickField(entry, 'accessor_key', 'accessorKey', 'key');
        if (typeof accessor !== 'string' || accessor.trim().length === 0) {
          return null;
        }
        const header =
          extractTextValue(pickField(entry, 'header', 'title', 'label')) || humanizeId(accessor);
        return {
          header,
          accessor_key: accessor,
        };
      })
      .filter(
        (entry): entry is { header: string; accessor_key: string } => Boolean(entry),
      );

    if (normalized.length > 0) {
      return normalized;
    }
  }

  const sample = rows[0];
  if (!sample) {
    return [];
  }

  return Object.keys(sample).map((key) => ({
    header: humanizeId(key),
    accessor_key: key,
  }));
}

function convertLayoutSection(section: Record<string, unknown>, index: number): Component | null {
  const sectionTitle =
    extractTextValue(pickField(section, 'title')) || `Section ${index + 1}`;
  const sectionTypeRaw = pickField(section, 'type', 'section_type', 'sectionType');
  const sectionType =
    typeof sectionTypeRaw === 'string' ? sectionTypeRaw.trim().toLowerCase() : '';

  if (sectionType === 'alert') {
    return {
      type: 'alert',
      title: sectionTitle,
      description:
        extractTextValue(pickField(section, 'message', 'description')) || undefined,
      variant: normalizeAlertVariant(pickField(section, 'severity', 'variant')),
    };
  }

  if (sectionType === 'stats') {
    const statsRaw = pickField(section, 'stats');
    const stats = Array.isArray(statsRaw)
      ? statsRaw.filter((entry): entry is Record<string, unknown> => isRecord(entry))
      : [];
    const cards = stats.map((entry, statIndex) => {
      const label =
        extractTextValue(pickField(entry, 'label', 'title')) || `Stat ${statIndex + 1}`;
      const value =
        extractTextValue(pickField(entry, 'value')) || stringifyDisplayValue(pickField(entry, 'value'));
      const statusRaw = pickField(entry, 'status');
      const status =
        typeof statusRaw === 'string' && statusRaw.trim().length > 0 ? statusRaw : null;
      const content: Component[] = [
        { type: 'text', content: value, variant: 'h3' },
      ];
      if (status) {
        content.push({
          type: 'badge',
          label: humanizeId(status),
          variant: normalizeBadgeVariant(status),
        });
      }
      return {
        type: 'card',
        title: label,
        content,
      } satisfies Component;
    });

    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'grid',
          columns: Math.min(Math.max(cards.length, 1), 3),
          children: cards.length > 0 ? cards : [{ type: 'text', content: '(No stats)', variant: 'caption' }],
        },
      ],
    };
  }

  if (sectionType === 'text') {
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'text',
          content:
            extractTextValue(pickField(section, 'text', 'message', 'description')) || '(No content)',
          variant: 'body',
        },
      ],
    };
  }

  if (sectionType === 'table') {
    const rows = normalizeTableRows(pickField(section, 'rows', 'data'));
    const columns = normalizeTableColumns(pickField(section, 'columns'), rows);
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'table',
          columns,
          data: rows,
        },
      ],
    };
  }

  if (sectionType === 'chart') {
    const rows = normalizeTableRows(pickField(section, 'data'));
    const xKeyRaw = pickField(section, 'x_key', 'xKey');
    const xKey =
      typeof xKeyRaw === 'string' && xKeyRaw.trim().length > 0
        ? xKeyRaw
        : Object.keys(rows[0] ?? {})[0] ?? 'x';
    const yKeysRaw = pickField(section, 'y_keys', 'yKeys');
    const yKeys = Array.isArray(yKeysRaw)
      ? yKeysRaw.filter((entry): entry is string => typeof entry === 'string' && entry.trim().length > 0)
      : [];
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'chart',
          title: undefined,
          kind: normalizeChartKind(pickField(section, 'chart_type', 'chartType', 'type')),
          data: rows,
          x_key: xKey,
          y_keys: yKeys,
        },
      ],
    };
  }

  if (sectionType === 'key_value') {
    const pairsRaw = pickField(section, 'pairs');
    const pairs = Array.isArray(pairsRaw)
      ? pairsRaw
          .filter((entry): entry is Record<string, unknown> => isRecord(entry))
          .map((entry) => ({
            key:
              extractTextValue(pickField(entry, 'key', 'label')) || 'Key',
            value:
              extractTextValue(pickField(entry, 'value')) || stringifyDisplayValue(pickField(entry, 'value')),
          }))
      : [];
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'key_value',
          pairs,
        },
      ],
    };
  }

  if (sectionType === 'list') {
    const itemsRaw = pickField(section, 'items');
    const items = Array.isArray(itemsRaw)
      ? itemsRaw.map((entry) => stringifyDisplayValue(entry)).filter((entry) => entry.trim().length > 0)
      : [];
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'list',
          items,
          ordered: pickField(section, 'ordered') === true,
        },
      ],
    };
  }

  if (sectionType === 'code_block') {
    return {
      type: 'card',
      title: sectionTitle,
      content: [
        {
          type: 'code_block',
          code: extractTextValue(pickField(section, 'code')) || '',
          language:
            typeof pickField(section, 'language') === 'string'
              ? (pickField(section, 'language') as string)
              : undefined,
        },
      ],
    };
  }

  return {
    type: 'card',
    title: sectionTitle,
    content: [
      {
        type: 'text',
        content: `Unsupported section type: ${sectionType || 'unknown'}`,
        variant: 'caption',
      },
    ],
  };
}

function convertRenderFormField(
  field: Record<string, unknown>,
  index: number,
): Component | null {
  const typeRaw = pickField(field, 'type');
  const type = typeof typeRaw === 'string' ? typeRaw.trim().toLowerCase() : '';
  const label =
    extractTextValue(pickField(field, 'label', 'title')) || `Field ${index + 1}`;
  const nameRaw = pickField(field, 'name', 'id');
  const name =
    typeof nameRaw === 'string' && nameRaw.trim().length > 0 ? nameRaw : `field_${index + 1}`;
  const required = pickField(field, 'required') === true;

  if (
    type === 'text' ||
    type === 'email' ||
    type === 'password' ||
    type === 'tel' ||
    type === 'url'
  ) {
    return {
      type: 'text_input',
      name,
      label,
      input_type: type === 'text' ? 'text' : (type as 'email' | 'password' | 'tel' | 'url'),
      placeholder:
        extractTextValue(pickField(field, 'placeholder', 'hint')) || undefined,
      required,
    };
  }

  if (type === 'textarea') {
    return {
      type: 'textarea',
      name,
      label,
      placeholder:
        extractTextValue(pickField(field, 'placeholder', 'hint')) || undefined,
      required,
      rows:
        typeof pickField(field, 'rows') === 'number'
          ? (pickField(field, 'rows') as number)
          : undefined,
    };
  }

  if (type === 'select') {
    const optionsRaw = pickField(field, 'options');
    const options = Array.isArray(optionsRaw)
      ? optionsRaw
          .filter((entry): entry is Record<string, unknown> => isRecord(entry))
          .map((entry) => {
            const value = pickField(entry, 'value');
            const labelValue = pickField(entry, 'label');
            return {
              value: typeof value === 'string' ? value : String(value ?? ''),
              label:
                extractTextValue(labelValue) ||
                (typeof labelValue === 'string' ? labelValue : String(value ?? '')),
            };
          })
          .filter((entry) => entry.value.trim().length > 0)
      : [];

    return {
      type: 'select',
      name,
      label,
      required,
      options,
    };
  }

  if (type === 'number') {
    return {
      type: 'number_input',
      name,
      label,
      required,
      min: typeof pickField(field, 'min') === 'number' ? (pickField(field, 'min') as number) : undefined,
      max: typeof pickField(field, 'max') === 'number' ? (pickField(field, 'max') as number) : undefined,
      step: typeof pickField(field, 'step') === 'number' ? (pickField(field, 'step') as number) : undefined,
    };
  }

  if (type === 'date') {
    return {
      type: 'date_input',
      name,
      label,
      required,
    };
  }

  if (type === 'switch' || type === 'checkbox') {
    return {
      type: 'switch',
      name,
      label,
      default_checked: pickField(field, 'default_checked', 'defaultChecked') === true,
    };
  }

  if (type === 'slider') {
    return {
      type: 'slider',
      name,
      label,
      min: typeof pickField(field, 'min') === 'number' ? (pickField(field, 'min') as number) : undefined,
      max: typeof pickField(field, 'max') === 'number' ? (pickField(field, 'max') as number) : undefined,
      step: typeof pickField(field, 'step') === 'number' ? (pickField(field, 'step') as number) : undefined,
    };
  }

  return null;
}

function extractSurfaceFromRenderToolInvocation(
  toolName: string,
  response: unknown,
): SurfaceSnapshot | null {
  if (!isRecord(response)) {
    return null;
  }

  if (toolName === 'render_alert') {
    const title = extractTextValue(pickField(response, 'title')) || 'Alert';
    const description =
      extractTextValue(pickField(response, 'description', 'message')) || undefined;
    return {
      surfaceId: typeof pickField(response, 'surface_id', 'surfaceId') === 'string'
        ? (pickField(response, 'surface_id', 'surfaceId') as string)
        : 'main',
      dataModel: {},
      components: [
        {
          type: 'alert',
          title,
          description,
          variant: normalizeAlertVariant(pickField(response, 'variant', 'severity')),
        },
      ],
    };
  }

  if (toolName === 'render_card') {
    const actionsRaw = pickField(response, 'actions');
    const footer = Array.isArray(actionsRaw)
      ? actionsRaw.reduce<Component[]>((items, entry) => {
          if (!isRecord(entry)) {
            return items;
          }
          const actionId = pickField(entry, 'action_id', 'actionId');
          const label = pickField(entry, 'label', 'title');
          if (typeof actionId !== 'string' || actionId.trim().length === 0) {
            return items;
          }
          items.push({
            type: 'button',
            action_id: actionId,
            label:
              extractTextValue(label) ||
              (typeof label === 'string' ? label : humanizeId(actionId)),
            variant: normalizeButtonVariant(pickField(entry, 'variant')),
          });
          return items;
        }, [])
      : undefined;

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'card',
          title: extractTextValue(pickField(response, 'title')) || 'Card',
          description:
            extractTextValue(pickField(response, 'description', 'subtitle')) || undefined,
          content: [
            {
              type: 'text',
              content:
                extractTextValue(pickField(response, 'content', 'message')) || '(No content)',
              variant: 'body',
            },
          ],
          footer: footer && footer.length > 0 ? footer : undefined,
        },
      ],
    };
  }

  if (toolName === 'render_table') {
    const rows = normalizeTableRows(pickField(response, 'data', 'rows'));
    const columns = normalizeTableColumns(pickField(response, 'columns'), rows);
    const tableComponent: Component = {
      type: 'table',
      columns,
      data: rows,
    };

    const title = extractTextValue(pickField(response, 'title'));
    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: title
        ? [
            {
              type: 'card',
              title,
              content: [tableComponent],
            },
          ]
        : [tableComponent],
    };
  }

  if (toolName === 'render_chart') {
    const rows = normalizeTableRows(pickField(response, 'data'));
    const xKeyRaw = pickField(response, 'x_key', 'xKey');
    const xKey =
      typeof xKeyRaw === 'string' && xKeyRaw.trim().length > 0
        ? xKeyRaw
        : Object.keys(rows[0] ?? {})[0] ?? 'x';
    const yKeysRaw = pickField(response, 'y_keys', 'yKeys');
    const yKeys = Array.isArray(yKeysRaw)
      ? yKeysRaw.filter((entry): entry is string => typeof entry === 'string' && entry.trim().length > 0)
      : [];

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'chart',
          title: extractTextValue(pickField(response, 'title')) || undefined,
          kind: normalizeChartKind(pickField(response, 'type', 'chart_type', 'chartType')),
          data: rows,
          x_key: xKey,
          y_keys: yKeys,
        },
      ],
    };
  }

  if (toolName === 'render_toast') {
    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'toast',
          message:
            extractTextValue(pickField(response, 'message', 'description')) || 'Notification',
          variant: normalizeAlertVariant(pickField(response, 'variant', 'severity')),
          duration:
            typeof pickField(response, 'duration') === 'number'
              ? (pickField(response, 'duration') as number)
              : undefined,
          dismissible:
            typeof pickField(response, 'dismissible') === 'boolean'
              ? (pickField(response, 'dismissible') as boolean)
              : undefined,
        },
      ],
    };
  }

  if (toolName === 'render_confirm') {
    const footer: Component[] = [];
    const cancelActionRaw = pickField(response, 'cancel_action', 'cancelAction');
    const cancelLabelRaw = pickField(response, 'cancel_label', 'cancelLabel');
    if (typeof cancelActionRaw === 'string' && cancelActionRaw.trim().length > 0) {
      footer.push({
        type: 'button',
        label:
          extractTextValue(cancelLabelRaw) ||
          (typeof cancelLabelRaw === 'string' ? cancelLabelRaw : '') ||
          'Cancel',
        action_id: cancelActionRaw,
        variant: 'ghost',
      });
    }

    const confirmActionRaw = pickField(response, 'confirm_action', 'confirmAction');
    if (typeof confirmActionRaw === 'string' && confirmActionRaw.trim().length > 0) {
      const confirmLabelRaw = pickField(response, 'confirm_label', 'confirmLabel');
      footer.push({
        type: 'button',
        label:
          extractTextValue(confirmLabelRaw) ||
          (typeof confirmLabelRaw === 'string' ? confirmLabelRaw : '') ||
          'Confirm',
        action_id: confirmActionRaw,
        variant: pickField(response, 'destructive') === true ? 'danger' : 'primary',
      });
    }

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'modal',
          title: extractTextValue(pickField(response, 'title')) || 'Confirm Action',
          content: [
            {
              type: 'text',
              content:
                extractTextValue(pickField(response, 'message', 'description')) ||
                '(No confirmation message provided)',
              variant: 'body',
            },
          ],
          footer: footer.length > 0 ? footer : undefined,
          size: 'small',
          closable: true,
        },
      ],
    };
  }

  if (toolName === 'render_progress') {
    const stepsRaw = pickField(response, 'steps');
    const steps = Array.isArray(stepsRaw)
      ? stepsRaw
          .filter((entry): entry is Record<string, unknown> => isRecord(entry))
          .map((entry) => {
            const label = extractTextValue(pickField(entry, 'label'));
            if (!label) {
              return null;
            }
            if (pickField(entry, 'completed') === true) {
              return `Done: ${label}`;
            }
            if (pickField(entry, 'current') === true) {
              return `Current: ${label}`;
            }
            return `Pending: ${label}`;
          })
          .filter((entry): entry is string => Boolean(entry))
      : [];

    const valueRaw = pickField(response, 'value');
    const value =
      typeof valueRaw === 'number' && Number.isFinite(valueRaw)
        ? Math.max(0, Math.min(100, valueRaw))
        : 0;
    const content: Component[] = [
      {
        type: 'progress',
        value,
        label: `${value}%`,
      },
    ];

    const description =
      extractTextValue(pickField(response, 'description', 'message')) || undefined;
    if (description) {
      content.push({
        type: 'text',
        content: description,
        variant: 'caption',
      });
    }
    if (steps.length > 0) {
      content.push({
        type: 'list',
        items: steps,
        ordered: true,
      });
    }

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'card',
          title: extractTextValue(pickField(response, 'title')) || 'Progress',
          content,
        },
      ],
    };
  }

  if (toolName === 'render_form') {
    const fieldsRaw = pickField(response, 'fields');
    const fields = Array.isArray(fieldsRaw)
      ? fieldsRaw
          .filter((entry): entry is Record<string, unknown> => isRecord(entry))
          .map((entry, index) => convertRenderFormField(entry, index))
          .filter((entry): entry is Component => Boolean(entry))
      : [];
    const submitActionRaw = pickField(response, 'submit_action', 'submitAction');
    const submitAction =
      typeof submitActionRaw === 'string' && submitActionRaw.trim().length > 0
        ? submitActionRaw
        : 'form_submit';
    const submitLabelRaw = pickField(response, 'submit_label', 'submitLabel');
    const submitLabel =
      extractTextValue(submitLabelRaw) ||
      (typeof submitLabelRaw === 'string' ? submitLabelRaw : '') ||
      humanizeId(submitAction);

    return {
      surfaceId: typeof pickField(response, 'surface_id', 'surfaceId') === 'string'
        ? (pickField(response, 'surface_id', 'surfaceId') as string)
        : 'main',
      dataModel: {},
      components: [
        {
          type: 'card',
          title:
            extractTextValue(pickField(response, 'title')) || 'Form',
          description:
            extractTextValue(pickField(response, 'description', 'subtitle')) || undefined,
          content: [
            ...fields,
            {
              type: 'button',
              label: submitLabel,
              action_id: submitAction,
              variant: 'primary',
            },
          ],
        },
      ],
    };
  }

  if (toolName === 'render_modal') {
    const footer: Component[] = [];
    const cancelLabelRaw = pickField(response, 'cancel_label', 'cancelLabel');
    if (typeof cancelLabelRaw === 'string' && cancelLabelRaw.trim().length > 0) {
      footer.push({
        type: 'button',
        label: cancelLabelRaw,
        action_id:
          typeof pickField(response, 'cancel_action', 'cancelAction') === 'string'
            ? (pickField(response, 'cancel_action', 'cancelAction') as string)
            : 'modal_cancel',
        variant: 'secondary',
      });
    }
    const confirmLabelRaw = pickField(response, 'confirm_label', 'confirmLabel');
    if (typeof confirmLabelRaw === 'string' && confirmLabelRaw.trim().length > 0) {
      footer.push({
        type: 'button',
        label: confirmLabelRaw,
        action_id:
          typeof pickField(response, 'confirm_action', 'confirmAction') === 'string'
            ? (pickField(response, 'confirm_action', 'confirmAction') as string)
            : 'modal_confirm',
        variant: 'primary',
      });
    }

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'modal',
          title: extractTextValue(pickField(response, 'title')) || 'Modal',
          content: [
            {
              type: 'text',
              content:
                extractTextValue(pickField(response, 'message', 'description')) ||
                '(No modal content)',
              variant: 'body',
            },
          ],
          footer: footer.length > 0 ? footer : undefined,
          size: normalizeModalSize(pickField(response, 'size')),
          closable: pickField(response, 'closable') !== false,
        },
      ],
    };
  }

  if (toolName === 'render_layout') {
    const sectionsRaw = pickField(response, 'sections');
    const sections = Array.isArray(sectionsRaw)
      ? sectionsRaw
          .filter((entry): entry is Record<string, unknown> => isRecord(entry))
          .map((entry, index) => convertLayoutSection(entry, index))
          .filter((entry): entry is Component => Boolean(entry))
      : [];
    const children: Component[] = [];
    const title = extractTextValue(pickField(response, 'title'));
    const description = extractTextValue(pickField(response, 'description'));
    if (title) {
      children.push({
        type: 'text',
        content: title,
        variant: 'h2',
      });
    }
    if (description) {
      children.push({
        type: 'text',
        content: description,
        variant: 'caption',
      });
    }
    children.push(...sections);

    return {
      surfaceId: normalizeSurfaceIdFromResponse(response),
      dataModel: {},
      components: [
        {
          type: 'stack',
          direction: 'vertical',
          gap: 20,
          children,
        },
      ],
    };
  }

  return null;
}

function extractEventText(event: Record<string, unknown>): string {
  const eventType = typeof pickField(event, 'type') === 'string'
    ? (pickField(event, 'type') as string).trim().toUpperCase()
    : '';
  if (
    eventType === 'TEXT_MESSAGE_CONTENT'
    || eventType === 'TEXT_MESSAGE_DELTA'
    || eventType === 'TEXT_MESSAGE_CHUNK'
  ) {
    const delta = pickField(event, 'delta', 'content');
    if (typeof delta === 'string') {
      return delta;
    }
  }

  const llmResponse = pickField(event, 'llm_response', 'llmResponse');
  const nestedContent = isRecord(llmResponse) ? pickField(llmResponse, 'content') : undefined;
  const content = pickField(event, 'content') ?? nestedContent;
  if (!isRecord(content)) {
    return '';
  }

  const parts = pickField(content, 'parts');
  if (!Array.isArray(parts)) {
    return '';
  }

  const snippets: string[] = [];
  for (const part of parts) {
    if (!isRecord(part)) {
      continue;
    }
    const text = pickField(part, 'text');
    if (typeof text === 'string') {
      snippets.push(text);
    }
  }

  return snippets.join(' ').trim();
}

type ToolResponseSource = 'call' | 'response';

type AgUiToolCallBuffer = {
  toolCallName?: string;
  delta: string;
};

type AgUiRuntimeContext = {
  toolCallNames: Map<string, string>;
  toolCallArgs: Map<string, AgUiToolCallBuffer>;
};

function normalizeEventType(event: Record<string, unknown>): string {
  const type = pickField(event, 'type');
  return typeof type === 'string' ? type.trim().toUpperCase() : '';
}

function resolveAgUiToolCallName(
  event: Record<string, unknown>,
  context?: AgUiRuntimeContext,
): string {
  const directName = pickField(event, 'toolCallName', 'tool_call_name', 'toolName', 'tool_name', 'name');
  if (typeof directName === 'string' && directName.trim().length > 0) {
    return directName;
  }

  const toolCallId = pickField(event, 'toolCallId', 'tool_call_id');
  if (typeof toolCallId === 'string' && context?.toolCallNames.has(toolCallId)) {
    return context.toolCallNames.get(toolCallId) as string;
  }

  if (typeof toolCallId === 'string' && toolCallId.trim().length > 0) {
    return toolCallId;
  }

  return 'tool_response';
}

function collectResponseSurfacesByTool(
  toolResponses: Array<{ name: string; response: unknown; source: ToolResponseSource }>,
): Map<string, SurfaceSnapshot> {
  const surfaces = new Map<string, SurfaceSnapshot>();

  for (const toolResponse of toolResponses) {
    if (toolResponse.source !== 'response') {
      continue;
    }

    const surface = extractSurfaceFromToolResponse(toolResponse.response);
    if (!surface) {
      continue;
    }

    surfaces.set(toolResponse.name, surface);
  }

  return surfaces;
}

function decodeJsonPointer(path: string): string[] | null {
  if (path === '') {
    return [];
  }
  if (!path.startsWith('/')) {
    return null;
  }
  return path
    .slice(1)
    .split('/')
    .map((segment) => segment.replace(/~1/g, '/').replace(/~0/g, '~'));
}

function cloneJsonValue<T>(value: T): T {
  if (typeof structuredClone === 'function') {
    return structuredClone(value);
  }
  return JSON.parse(JSON.stringify(value)) as T;
}

function getPatchParent(
  root: unknown,
  segments: string[],
): { parent: Record<string, unknown> | unknown[]; key: string } | null {
  if (segments.length === 0) {
    return null;
  }

  let current: unknown = root;
  for (let index = 0; index < segments.length - 1; index += 1) {
    const segment = segments[index];
    if (Array.isArray(current)) {
      const nextIndex = Number(segment);
      if (!Number.isInteger(nextIndex) || nextIndex < 0 || nextIndex >= current.length) {
        return null;
      }
      current = current[nextIndex];
      continue;
    }
    if (!isRecord(current) || !(segment in current)) {
      return null;
    }
    current = current[segment];
  }

  if (Array.isArray(current) || isRecord(current)) {
    return { parent: current, key: segments[segments.length - 1] };
  }
  return null;
}

function getValueAtJsonPointer(root: unknown, path: string): unknown {
  const segments = decodeJsonPointer(path);
  if (segments === null) {
    return undefined;
  }
  let current: unknown = root;
  for (const segment of segments) {
    if (Array.isArray(current)) {
      const index = Number(segment);
      if (!Number.isInteger(index) || index < 0 || index >= current.length) {
        return undefined;
      }
      current = current[index];
      continue;
    }
    if (!isRecord(current) || !(segment in current)) {
      return undefined;
    }
    current = current[segment];
  }
  return current;
}

function applySingleJsonPatch(root: Record<string, unknown>, operation: JsonPatchOperation): void {
  const op = typeof operation.op === 'string' ? operation.op : null;
  const path = typeof operation.path === 'string' ? operation.path : null;
  if (!op || path === null) {
    return;
  }

  if (path === '') {
    if (op === 'replace' || op === 'add') {
      const value = isRecord(operation.value) ? operation.value : { value: operation.value ?? null };
      for (const key of Object.keys(root)) {
        delete root[key];
      }
      Object.assign(root, cloneJsonValue(value));
    } else if (op === 'remove') {
      for (const key of Object.keys(root)) {
        delete root[key];
      }
    }
    return;
  }

  const segments = decodeJsonPointer(path);
  if (!segments) {
    return;
  }
  const target = getPatchParent(root, segments);
  if (!target) {
    return;
  }

  if (op === 'move' || op === 'copy') {
    if (typeof operation.from !== 'string') {
      return;
    }
    const fromValue = getValueAtJsonPointer(root, operation.from);
    if (fromValue === undefined) {
      return;
    }
    const movedValue = cloneJsonValue(fromValue);
    applySingleJsonPatch(root, { op: 'add', path, value: movedValue });
    if (op === 'move') {
      applySingleJsonPatch(root, { op: 'remove', path: operation.from });
    }
    return;
  }

  if (op === 'test') {
    return;
  }

  if (Array.isArray(target.parent)) {
    const array = target.parent;
    const index = target.key === '-' ? array.length : Number(target.key);
    if (!Number.isInteger(index) || index < 0 || index > array.length) {
      return;
    }
    if (op === 'add') {
      array.splice(index, 0, cloneJsonValue(operation.value));
      return;
    }
    if (op === 'replace') {
      if (index >= array.length) {
        return;
      }
      array[index] = cloneJsonValue(operation.value);
      return;
    }
    if (op === 'remove') {
      if (index >= array.length) {
        return;
      }
      array.splice(index, 1);
    }
    return;
  }

  if (op === 'add' || op === 'replace') {
    target.parent[target.key] = cloneJsonValue(operation.value);
    return;
  }
  if (op === 'remove') {
    delete target.parent[target.key];
  }
}

function applyActivityPatch(
  content: Record<string, unknown>,
  patch: JsonPatchOperation[],
): Record<string, unknown> {
  const next = cloneJsonValue(content);
  for (const operation of patch) {
    applySingleJsonPatch(next, operation);
  }
  return next;
}

function extractAgUiActivityMessage(event: Record<string, unknown>): AgUiActivityMessage | null {
  if (normalizeEventType(event) !== 'ACTIVITY_SNAPSHOT') {
    return null;
  }

  const messageId = pickField(event, 'messageId', 'message_id', 'id');
  if (typeof messageId !== 'string' || messageId.trim().length === 0) {
    return null;
  }

  const activityType = pickField(event, 'activityType', 'activity_type', 'name');
  if (typeof activityType !== 'string' || activityType.trim().length === 0) {
    return null;
  }

  const contentRaw = pickField(event, 'content');
  const content = isRecord(contentRaw)
    ? contentRaw
    : { value: contentRaw ?? null };
  const replaceRaw = pickField(event, 'replace');

  return {
    messageId,
    activityType,
    content,
    replace: replaceRaw === false ? false : true,
  };
}

function extractAgUiActivityDelta(
  event: Record<string, unknown>,
): { messageId: string; activityType: string; patch: JsonPatchOperation[] } | null {
  if (normalizeEventType(event) !== 'ACTIVITY_DELTA') {
    return null;
  }

  const messageId = pickField(event, 'messageId', 'message_id', 'id');
  const activityType = pickField(event, 'activityType', 'activity_type', 'name');
  const patch = pickField(event, 'patch');
  if (
    typeof messageId !== 'string'
    || messageId.trim().length === 0
    || typeof activityType !== 'string'
    || activityType.trim().length === 0
    || !Array.isArray(patch)
  ) {
    return null;
  }

  return {
    messageId,
    activityType,
    patch: patch.filter(isRecord),
  };
}

function upsertAgUiActivityMessage(
  current: AgUiActivityMessage[],
  next: AgUiActivityMessage,
): AgUiActivityMessage[] {
  const existingIndex = current.findIndex((entry) => entry.messageId === next.messageId);
  if (existingIndex === -1) {
    return [...current, next];
  }
  if (!next.replace) {
    return current;
  }
  const updated = current.slice();
  updated[existingIndex] = next;
  return updated;
}

function applyAgUiActivityDelta(
  current: AgUiActivityMessage[],
  delta: { messageId: string; activityType: string; patch: JsonPatchOperation[] },
): AgUiActivityMessage[] {
  const existingIndex = current.findIndex((entry) => entry.messageId === delta.messageId);
  const existing = existingIndex === -1
    ? {
        messageId: delta.messageId,
        activityType: delta.activityType,
        content: {},
        replace: true,
      }
    : current[existingIndex];
  const updatedEntry: AgUiActivityMessage = {
    ...existing,
    activityType: delta.activityType,
    content: applyActivityPatch(existing.content, delta.patch),
  };
  if (existingIndex === -1) {
    return [...current, updatedEntry];
  }
  const updated = current.slice();
  updated[existingIndex] = updatedEntry;
  return updated;
}

function extractAgUiToolCallDelta(event: Record<string, unknown>): string | undefined {
  const delta = pickField(event, 'delta', 'args', 'arguments', 'payload', 'data');
  if (typeof delta === 'string') {
    return delta;
  }

  if (delta === undefined) {
    return undefined;
  }

  try {
    return JSON.stringify(delta);
  } catch {
    return String(delta);
  }
}

function parseAgUiToolCallPayload(delta: string): unknown | undefined {
  const parsed = parseMaybeJson(delta);
  const trimmed = delta.trim();
  if (typeof parsed === 'string' && (trimmed.startsWith('{') || trimmed.startsWith('['))) {
    return undefined;
  }
  return parsed;
}

function extractToolResponses(
  event: Record<string, unknown>,
  agUiContext?: AgUiRuntimeContext,
): Array<{ name: string; response: unknown; source: ToolResponseSource }> {
  const eventType = normalizeEventType(event);
  if (eventType === 'TOOL_CALL_ARGS' || eventType === 'TOOL_CALL_CHUNK') {
    const toolCallId = pickField(event, 'toolCallId', 'tool_call_id');
    const delta = extractAgUiToolCallDelta(event);
    const resolvedName = resolveAgUiToolCallName(event, agUiContext);
    if (typeof toolCallId === 'string' && agUiContext) {
      const existing = agUiContext.toolCallArgs.get(toolCallId) ?? { delta: '' };
      if (resolvedName.trim().length > 0 && resolvedName !== 'tool_response') {
        existing.toolCallName = resolvedName;
      }
      if (delta) {
        existing.delta += delta;
      }
      agUiContext.toolCallArgs.set(toolCallId, existing);
      const parsed = parseAgUiToolCallPayload(existing.delta);
      if (parsed === undefined) {
        return [];
      }
      return [
        {
          name: existing.toolCallName ?? resolvedName,
          response: parsed,
          source: 'call',
        },
      ];
    }

    if (delta !== undefined) {
      const parsed = parseAgUiToolCallPayload(delta);
      if (parsed === undefined) {
        return [];
      }
      return [
        {
          name: resolvedName,
          response: parsed,
          source: 'call',
        },
      ];
    }
  }

  if (eventType === 'TOOL_CALL_RESULT') {
    const content = pickField(event, 'content', 'result', 'payload', 'data');
    if (content !== undefined) {
      return [
        {
          name: resolveAgUiToolCallName(event, agUiContext),
          response: parseMaybeJson(content),
          source: 'response',
        },
      ];
    }
  }

  const llmResponse = pickField(event, 'llm_response', 'llmResponse');
  const nestedContent = isRecord(llmResponse) ? pickField(llmResponse, 'content') : undefined;
  const content = pickField(event, 'content') ?? nestedContent;
  if (!isRecord(content)) {
    return [];
  }

  const parts = pickField(content, 'parts');
  if (!Array.isArray(parts)) {
    return [];
  }

  const responses: Array<{ name: string; response: unknown; source: ToolResponseSource }> = [];

  for (const part of parts) {
    if (!isRecord(part)) {
      continue;
    }

    const functionCallWrapped = pickField(part, 'functionCall', 'function_call', 'toolCall', 'tool_call');
    if (isRecord(functionCallWrapped)) {
      const callName = pickField(functionCallWrapped, 'name', 'toolName', 'tool_name');
      const callArgs = pickField(functionCallWrapped, 'args', 'arguments', 'parameters', 'input', 'payload');
      if (typeof callName === 'string') {
        responses.push({
          name: callName,
          response: parseMaybeJson(callArgs ?? functionCallWrapped),
          source: 'call',
        });
      }
      continue;
    }

    // adk-core FunctionCall part shape is untagged: { name, args, id? }
    const directCallName = pickField(part, 'name');
    const directCallArgs = pickField(part, 'args', 'arguments', 'parameters', 'input', 'payload');
    if (typeof directCallName === 'string' && directCallArgs !== undefined) {
      responses.push({
        name: directCallName,
        response: parseMaybeJson(directCallArgs),
        source: 'call',
      });
      continue;
    }

    const functionResponse = pickField(part, 'functionResponse', 'function_response', 'toolResponse', 'tool_response');
    if (!isRecord(functionResponse)) {
      continue;
    }

    const name =
      (typeof pickField(functionResponse, 'name', 'toolName', 'tool_name') === 'string'
        ? (pickField(functionResponse, 'name', 'toolName', 'tool_name') as string)
        : 'tool_response') || 'tool_response';

    responses.push({
      name,
      response:
        parseMaybeJson(
          pickField(functionResponse, 'response', 'result', 'output', 'payload', 'data') ??
            functionResponse,
        ),
      source: 'response',
    });
  }

  // Also accept event-level tool response envelopes.
  const eventToolResponse = pickField(event, 'tool_response', 'toolResponse');
  if (isRecord(eventToolResponse)) {
    const nameRaw = pickField(eventToolResponse, 'name', 'toolName', 'tool_name');
    const name = typeof nameRaw === 'string' ? nameRaw : 'tool_response';
    responses.push({
      name,
      response:
        parseMaybeJson(
          pickField(eventToolResponse, 'response', 'result', 'output', 'payload', 'data') ??
            eventToolResponse,
        ),
      source: 'response',
    });
  }

  const eventToolResponses = pickField(event, 'tool_responses', 'toolResponses');
  if (Array.isArray(eventToolResponses)) {
    for (const item of eventToolResponses) {
      if (!isRecord(item)) {
        continue;
      }
      const nameRaw = pickField(item, 'name', 'toolName', 'tool_name');
      const name = typeof nameRaw === 'string' ? nameRaw : 'tool_response';
      responses.push({
        name,
        response: parseMaybeJson(pickField(item, 'response', 'result', 'output', 'payload', 'data') ?? item),
        source: 'response',
      });
    }
  }

  return responses;
}

function extractEventError(event: Record<string, unknown>): string | null {
  const eventType = normalizeEventType(event);
  const candidates = [
    (eventType === 'RUN_ERROR' || eventType === 'ERROR')
      ? pickField(event, 'message', 'error_message', 'errorMessage')
      : undefined,
    pickField(event, 'error_message', 'errorMessage'),
    isRecord(pickField(event, 'llm_response', 'llmResponse'))
      ? pickField(pickField(event, 'llm_response', 'llmResponse') as Record<string, unknown>, 'error_message', 'errorMessage')
      : undefined,
  ];

  for (const candidate of candidates) {
    if (typeof candidate === 'string' && candidate.trim().length > 0) {
      return candidate.trim();
    }
  }

  return null;
}

function extractArtifactDelta(event: Record<string, unknown>): Array<{ name: string; version: number }> {
  const actions = pickField(event, 'actions');
  if (!isRecord(actions)) {
    return [];
  }

  const delta = pickField(actions, 'artifact_delta', 'artifactDelta');
  if (!isRecord(delta)) {
    return [];
  }

  const entries: Array<{ name: string; version: number }> = [];
  for (const [name, value] of Object.entries(delta)) {
    const numericVersion = typeof value === 'number' ? value : Number(value);
    if (!Number.isFinite(numericVersion)) {
      continue;
    }
    entries.push({ name, version: numericVersion });
  }

  return entries;
}

function toReadablePreview(raw: unknown, max = 10_000): string {
  let text: string;
  if (typeof raw === 'string') {
    text = raw;
  } else {
    try {
      text = JSON.stringify(raw);
    } catch {
      text = String(raw);
    }
  }
  return text.length > max ? `${text.slice(0, max)}...` : text;
}

function toPrettyJson(raw: unknown): string {
  try {
    return JSON.stringify(raw, null, 2);
  } catch {
    return String(raw);
  }
}

function trimForPanel(text: string, max = 200_000): string {
  if (text.length <= max) {
    return text;
  }
  return `${text.slice(0, max)}\n... (truncated)`;
}

function flattenSurfaceComponents(components: Component[]): Component[] {
  const result: Component[] = [];
  const stack: Component[] = [...components];

  while (stack.length > 0) {
    const node = stack.pop();
    if (!node) {
      continue;
    }
    result.push(node);

    const asRecord = node as Record<string, unknown>;
    const children = asRecord.children;
    const content = asRecord.content;
    const footer = asRecord.footer;
    const tabs = asRecord.tabs;

    if (Array.isArray(children)) {
      for (const entry of children) {
        if (isRecord(entry) && typeof pickField(entry as Record<string, unknown>, 'type') === 'string') {
          stack.push(entry as Component);
        }
      }
    }

    if (Array.isArray(content)) {
      for (const entry of content) {
        if (isRecord(entry) && typeof pickField(entry as Record<string, unknown>, 'type') === 'string') {
          stack.push(entry as Component);
        }
      }
    }

    if (Array.isArray(footer)) {
      for (const entry of footer) {
        if (isRecord(entry) && typeof pickField(entry as Record<string, unknown>, 'type') === 'string') {
          stack.push(entry as Component);
        }
      }
    }

    if (Array.isArray(tabs)) {
      for (const tab of tabs) {
        if (!isRecord(tab) || !Array.isArray(tab.content)) {
          continue;
        }
        for (const entry of tab.content) {
          if (isRecord(entry) && typeof pickField(entry as Record<string, unknown>, 'type') === 'string') {
            stack.push(entry as Component);
          }
        }
      }
    }
  }

  return result;
}

function isLowFidelitySurface(surface: SurfaceSnapshot): boolean {
  // AWP surfaces with HTML rendering are never low-fidelity — the HTML is the rich output
  if (surface.html) {
    return false;
  }

  const flat = flattenSurfaceComponents(surface.components);
  if (flat.length < 5) {
    return false;
  }

  const structuredTypes = new Set([
    'table',
    'chart',
    'card',
    'grid',
    'tabs',
    'modal',
    'alert',
    'key_value',
    'list',
  ]);

  const typeSet = new Set(flat.map((component) => component.type));
  const hasStructured = flat.some((component) => structuredTypes.has(component.type));
  if (hasStructured) {
    return false;
  }

  const textCount = flat.filter((component) => component.type === 'text').length;
  const buttonCount = flat.filter((component) => component.type === 'button').length;
  const textHeavy = textCount / flat.length >= 0.55;
  const lowVariety = typeSet.size <= 3;

  return textHeavy && lowVariety && buttonCount >= 1;
}

function buildAssistantFallback(
  lastRuntimeEvent: Record<string, unknown> | null,
  lastToolResponse: unknown | null,
  renderDetected: boolean,
): string | null {
  if (!lastRuntimeEvent && !lastToolResponse) {
    return null;
  }

  const lines: string[] = [];
  lines.push('No assistant text was emitted for this run.');

  if (!renderDetected) {
    lines.push('No renderable UI surface was detected in streamed tool payloads.');
  }

  if (lastToolResponse) {
    lines.push('');
    lines.push('Last tool payload:');
    lines.push(toPrettyJson(lastToolResponse));
  }

  if (lastRuntimeEvent) {
    lines.push('');
    lines.push('Last runtime event:');
    lines.push(toPrettyJson(lastRuntimeEvent));
  }

  return trimForPanel(lines.join('\n'));
}

function App() {
  const [selectedExample, setSelectedExample] = useState<ExampleTarget>(EXAMPLES[0]);
  const [selectedProtocol, setSelectedProtocol] = useState<UiProtocol>('a2ui');
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [surface, setSurface] = useState<SurfaceSnapshot | null>(null);
  const [promptInput, setPromptInput] = useState('');
  const [assistantText, setAssistantText] = useState('');
  const [activityMessages, setActivityMessages] = useState<AgUiActivityMessage[]>([]);
  const [eventLog, setEventLog] = useState<StreamLogEvent[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [isRetrying, setIsRetrying] = useState(false);
  const [statusText, setStatusText] = useState('Idle');
  const [runCount, setRunCount] = useState(0);
  const [toolHitCount, setToolHitCount] = useState(0);
  const [lastLatencyMs, setLastLatencyMs] = useState<number | null>(null);
  const [errorText, setErrorText] = useState<string | null>(null);
  const [capabilities, setCapabilities] = useState<ProtocolCapability[]>([]);
  const [capabilitiesError, setCapabilitiesError] = useState<string | null>(null);
  const agUiContextRef = useRef<{
    threadId?: string;
    lastRunId?: string;
    toolCallNames: Map<string, string>;
    toolCallArgs: Map<string, AgUiToolCallBuffer>;
  }>({
    toolCallNames: new Map(),
    toolCallArgs: new Map(),
  });
  const mcpAppsBridgeRef = useRef<McpAppsBridgeSessionState>({});

  const selectedProtocolMeta = useMemo(
    () => PROTOCOLS.find((protocol) => protocol.id === selectedProtocol) ?? PROTOCOLS[0],
    [selectedProtocol],
  );

  const capabilityForSelectedProtocol = useMemo(() => {
    return capabilities.find((capability) => normalizeProtocol(capability.protocol) === selectedProtocol);
  }, [capabilities, selectedProtocol]);

  const baseUrl = `http://localhost:${selectedExample.port}`;

  function appendEvent(kind: string, protocol: UiProtocol, payload: unknown, preview?: string) {
    const entry: StreamLogEvent = {
      id: Date.now() + Math.floor(Math.random() * 1000),
      at: nowLabel(),
      protocol,
      kind,
      preview: preview ?? toReadablePreview(payload),
      raw: payload,
    };

    setEventLog((current) => [entry, ...current].slice(0, MAX_EVENT_LOG));
  }

  async function postBridgeEndpoint(
    endpoint: string,
    protocol: UiProtocol,
    kind: string,
    body: Record<string, unknown>,
    preview: string,
  ): Promise<unknown> {
    const response = await fetch(`${baseUrl}${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    const responseText = await response.text();
    let parsed: unknown = responseText;
    if (responseText.trim().length > 0) {
      try {
        parsed = JSON.parse(responseText);
      } catch {
        parsed = responseText;
      }
    } else {
      parsed = { ok: response.ok, status: response.status };
    }

    if (!response.ok) {
      const message =
        isRecord(parsed) && typeof pickField(parsed, 'message', 'error') === 'string'
          ? (pickField(parsed, 'message', 'error') as string)
          : `Bridge request failed (${response.status}${response.statusText ? ` ${response.statusText}` : ''})`;
      throw new Error(message);
    }

    appendEvent(kind, protocol, parsed, preview);
    return parsed;
  }

  async function ensureMcpAppsBridgeInitialized(activeSessionId: string): Promise<boolean> {
    if (mcpAppsBridgeRef.current.sessionId === activeSessionId) {
      return false;
    }

    await postBridgeEndpoint(
      '/api/ui/initialize',
      'mcp_apps',
      'mcp_apps_initialize',
      buildMcpAppsInitializeBody(selectedExample.id, DEFAULT_USER_ID, activeSessionId),
      'ui/initialize',
    );
    mcpAppsBridgeRef.current = { sessionId: activeSessionId };
    return true;
  }

  function registerUiAction(action: UiEvent) {
    void (async () => {
      const activeSessionId = sessionId ?? makeSessionId();
      if (!sessionId) {
        setSessionId(activeSessionId);
      }

      const bridge = buildActionRequestBridge(selectedProtocol, action, {
        appName: selectedExample.id,
        userId: DEFAULT_USER_ID,
        sessionId: activeSessionId,
        surfaceId: surface?.surfaceId ?? 'main',
        threadId: agUiContextRef.current.threadId,
        parentRunId: agUiContextRef.current.lastRunId,
      });

      try {
        if (selectedProtocol === 'mcp_apps') {
          await ensureMcpAppsBridgeInitialized(activeSessionId);
          if (bridge.bridgeRequest) {
            await postBridgeEndpoint(
              bridge.bridgeRequest.endpoint,
              'mcp_apps',
              bridge.bridgeRequest.kind,
              bridge.bridgeRequest.body,
              bridge.bridgeRequest.preview,
            );
            if (bridge.bridgeRequest.skipModelRun) {
              setStatusText('MCP Apps model context updated');
              setErrorText(null);
              return;
            }
          }
        }

        await runPrompt(bridge.textMessage, {
          activeSessionId,
          skipPromptAugmentation: true,
          requestBody: bridge.requestBody,
          allowRetry: false,
          clearSurfaceOnStart: false,
          requestEventKind: 'ui_action_request',
          requestEventPayload: {
            action,
            outbound: bridge.outbound,
            message: bridge.textMessage,
          },
          requestPreview: `${bridge.preview}: ${action.action}`,
          initialStatusText: 'Submitting UI action...',
        });
      } catch (error) {
        const endpoint = bridge.bridgeRequest?.endpoint ?? '/api/ui/message';
        const message = normalizeRequestError(error, baseUrl, endpoint, selectedExample.port);
        setStatusText('Failed');
        setErrorText(message);
        appendEvent('error', selectedProtocol, { message }, message);
      }
    })();
  }

  async function loadCapabilities() {
    const endpoint = '/api/ui/capabilities';
    try {
      const response = await fetch(`${baseUrl}${endpoint}`);
      if (!response.ok) {
        throw new Error(`status ${response.status}${response.statusText ? ` ${response.statusText}` : ''}`);
      }
      const data = await response.json();
      const list = Array.isArray(data.protocols) ? (data.protocols as ProtocolCapability[]) : [];
      setCapabilities(list);
      setCapabilitiesError(null);
      appendEvent('capabilities', selectedProtocol, data, 'Loaded /api/ui/capabilities');
    } catch (error) {
      const message = normalizeRequestError(error, baseUrl, endpoint, selectedExample.port);
      setCapabilities([]);
      setCapabilitiesError(message);
    }
  }

  async function runPrompt(
    rawPrompt: string,
    options?: {
      retryAttempt?: number;
      activeSessionId?: string;
      retryMode?: 'no_surface' | 'quality';
      skipPromptAugmentation?: boolean;
      requestBody?: Record<string, unknown>;
      requestHeaders?: Record<string, string>;
      allowRetry?: boolean;
      clearSurfaceOnStart?: boolean;
      requestEventKind?: string;
      requestEventPayload?: unknown;
      requestPreview?: string;
      initialStatusText?: string;
    },
  ) {
    const retryAttempt = options?.retryAttempt ?? 0;
    const isAutoRetry = retryAttempt > 0;
    const prompt = rawPrompt.trim();
    if (!prompt || (isStreaming && !isAutoRetry)) {
      return;
    }

    const activeSessionId = options?.activeSessionId ?? sessionId ?? makeSessionId();
    if (!sessionId && !options?.activeSessionId) {
      setSessionId(activeSessionId);
    }

    setIsStreaming(true);
    setStatusText(options?.initialStatusText ?? (isAutoRetry ? 'Streaming UI (retrying)...' : 'Streaming...'));
    if (!isAutoRetry) {
      setIsRetrying(false);
    }
    setErrorText(null);
    if (!isAutoRetry) {
      setAssistantText('');
      setActivityMessages([]);
      setEventLog([]);
      if (options?.clearSurfaceOnStart !== false) {
        setSurface(null);
      }
      setToolHitCount(0);
    }

    const instructedPrompt = options?.skipPromptAugmentation
      ? prompt
      : `${prompt}\n\n${protocolInstruction(selectedProtocol)}${
          isAutoRetry ? `\n\n${retryInstruction(selectedProtocol, options?.retryMode ?? 'no_surface')}` : ''
        }`;
    const fetchedArtifactVersions = new Set<string>();
    let assistantTextCaptured = false;
    let renderDetected = false;
    let lowFidelityDetected = false;
    let latestSurface: SurfaceSnapshot | null = null;
    let lastRuntimeEventSeen: Record<string, unknown> | null = null;
    let lastToolResponseSeen: unknown | null = null;
    const startedAt = performance.now();
    const endpoint = '/api/run_sse';
    const requestPayload: Record<string, unknown> = {
      appName: selectedExample.id,
      userId: DEFAULT_USER_ID,
      sessionId: activeSessionId,
      uiProtocol: selectedProtocol,
      streaming: true,
      ...(options?.requestBody ?? {}),
    };
    const requestHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options?.requestHeaders ?? {}),
    };
    const hasAgUiInput = requestPayload.input !== undefined || requestPayload.agUiInput !== undefined;
    if (selectedProtocol === 'ag_ui') {
      requestPayload.uiTransport = 'protocol_native';
      requestHeaders['x-adk-ui-transport'] = 'protocol_native';

      if (!hasAgUiInput) {
        requestPayload.input = buildAgUiRuntimeInput(instructedPrompt, {
          sessionId: activeSessionId,
          surfaceId: surface?.surfaceId ?? 'main',
          threadId: agUiContextRef.current.threadId,
          parentRunId: agUiContextRef.current.lastRunId,
          activityMessages: buildPromptContextActivityMessages(prompt, {
            sessionId: activeSessionId,
            exampleId: selectedExample.id,
            retryAttempt,
            protocol: selectedProtocol,
          }),
          forwardedProps: {
            promptKind: isAutoRetry ? 'retry' : 'prompt',
            retryAttempt,
          },
        });
      }
    }
    if (
      requestPayload.newMessage === undefined
      && requestPayload.input === undefined
      && requestPayload.agUiInput === undefined
    ) {
      requestPayload.newMessage = {
        role: 'user',
        parts: [{ text: instructedPrompt }],
      };
    }
    const requestEventPayload =
      options?.requestEventPayload ??
      {
        appName: selectedExample.id,
        sessionId: activeSessionId,
        protocol: selectedProtocol,
        prompt,
      };

    try {
      if (selectedProtocol === 'mcp_apps') {
        await ensureMcpAppsBridgeInitialized(activeSessionId);
      }

      const response = await fetch(`${baseUrl}${endpoint}`, {
        method: 'POST',
        headers: requestHeaders,
        body: JSON.stringify(requestPayload),
      });

      if (!response.ok || !response.body) {
        throw new Error(
          `request failed (${response.status}${response.statusText ? ` ${response.statusText}` : ''})`,
        );
      }

      appendEvent(
        options?.requestEventKind ?? 'request',
        selectedProtocol,
        requestEventPayload,
        options?.requestPreview,
      );

      const decoder = new TextDecoder();
      const reader = response.body.getReader();
      let buffer = '';
      let receivedDoneSentinel = false;

      while (true) {
        const { done, value } = await reader.read();
        if (done) {
          break;
        }

        buffer += decoder.decode(value, { stream: true });
        const sseBatch = takeCompleteSseEvents(buffer);
        buffer = sseBatch.rest;

        for (const rawEvent of sseBatch.events) {
          const payload = extractSseEventPayload(rawEvent);
          if (!payload || payload === ':keep-alive') {
            continue;
          }
          if (payload === '[DONE]') {
            receivedDoneSentinel = true;
            break;
          }

          let parsed: unknown;
          try {
            parsed = JSON.parse(payload);
          } catch {
            appendEvent('parse_error', selectedProtocol, payload, 'Could not parse SSE JSON payload');
            continue;
          }

          let runtimeProtocol = selectedProtocol;
          let runtimeEvent = parsed;

          if (isRecord(parsed) && isRecord(parsed.event)) {
            runtimeProtocol = normalizeProtocol(parsed.ui_protocol) ?? selectedProtocol;
            runtimeEvent = parsed.event;
          }

          if (!isRecord(runtimeEvent)) {
            continue;
          }

          const runtimeEventType = normalizeEventType(runtimeEvent);
          const runtimeToolCallId = pickField(runtimeEvent, 'toolCallId', 'tool_call_id');
          if (runtimeProtocol === 'ag_ui') {
            const threadId = pickField(runtimeEvent, 'threadId', 'thread_id');
            const runId = pickField(runtimeEvent, 'runId', 'run_id');
            const toolCallName = pickField(runtimeEvent, 'toolCallName', 'tool_call_name', 'name');
            if (typeof threadId === 'string') {
              agUiContextRef.current.threadId = threadId;
            }
            if (typeof runId === 'string') {
              agUiContextRef.current.lastRunId = runId;
            }
            if (runtimeEventType === 'RUN_STARTED') {
              agUiContextRef.current.toolCallNames.clear();
              agUiContextRef.current.toolCallArgs.clear();
              setActivityMessages([]);
            }
            if (
              typeof runtimeToolCallId === 'string'
              && typeof toolCallName === 'string'
              && toolCallName.trim().length > 0
            ) {
              agUiContextRef.current.toolCallNames.set(runtimeToolCallId, toolCallName);
            }
          }

          lastRuntimeEventSeen = runtimeEvent;
          const runtimeError = extractEventError(runtimeEvent);
          const text = extractEventText(runtimeEvent);
          const activityMessage = extractAgUiActivityMessage(runtimeEvent);
          const activityDelta = extractAgUiActivityDelta(runtimeEvent);
          const toolResponses = extractToolResponses(runtimeEvent, agUiContextRef.current);
          const artifactDeltas = extractArtifactDelta(runtimeEvent);
          if (
            runtimeProtocol === 'ag_ui'
            && typeof runtimeToolCallId === 'string'
            && (runtimeEventType === 'TOOL_CALL_END' || runtimeEventType === 'TOOL_CALL_RESULT')
          ) {
            agUiContextRef.current.toolCallArgs.delete(runtimeToolCallId);
          }
          const turnCompleteRaw = pickField(runtimeEvent, 'turn_complete', 'turnComplete');
          const isTerminalEvent =
            turnCompleteRaw === true ||
            typeof runtimeError === 'string';

          if (isTerminalEvent && !runtimeError && !text && toolResponses.length === 0 && artifactDeltas.length === 0) {
            appendEvent('turn_complete', runtimeProtocol, runtimeEvent, 'Turn complete');
          }

          if (runtimeError) {
            assistantTextCaptured = true;
            setErrorText(runtimeError);
            setAssistantText((current) =>
              trimForPanel(
                `${current}${current ? '\n\n' : ''}[Runtime error]\n${runtimeError}`,
              ),
            );
            appendEvent('runtime_error', runtimeProtocol, runtimeEvent, runtimeError);
          }

          if (text) {
            assistantTextCaptured = true;
            setAssistantText((current) => `${current}${current ? '\n' : ''}${text}`);
          }

          if (activityMessage) {
            setActivityMessages((current) => upsertAgUiActivityMessage(current, activityMessage));
            appendEvent(
              `activity:${activityMessage.activityType.toLowerCase()}`,
              runtimeProtocol,
              runtimeEvent,
              toReadablePreview(activityMessage.content, 180),
            );
          }

          if (activityDelta) {
            setActivityMessages((current) => applyAgUiActivityDelta(current, activityDelta));
            appendEvent(
              `activity_delta:${activityDelta.activityType.toLowerCase()}`,
              runtimeProtocol,
              runtimeEvent,
              toReadablePreview(activityDelta.patch, 180),
            );
          }

          const responseToolResponses = toolResponses.filter((entry) => entry.source === 'response');
          const responseSurfaceByTool = collectResponseSurfacesByTool(toolResponses);
          if (responseToolResponses.length > 0) {
            setToolHitCount((count) => count + responseToolResponses.length);
          }

          for (const toolResponse of toolResponses) {
            if (toolResponse.source === 'call') {
              appendEvent(`tool_call:${toolResponse.name}`, runtimeProtocol, toolResponse.response);
              const responseSurface = responseSurfaceByTool.get(toolResponse.name);
              if (responseSurface?.protocol === 'mcp_apps') {
                continue;
              }
              // Tool call args for render_* tools contain the full component tree.
              // Try to extract a surface from the call args as well, since the SSE
              // stream may not include a separate functionResponse event.
              const maybeSurface =
                extractSurfaceFromToolResponse(toolResponse.response) ??
                extractSurfaceFromRenderToolInvocation(toolResponse.name, toolResponse.response);
              if (maybeSurface) {
                renderDetected = true;
                latestSurface = maybeSurface;
                lowFidelityDetected = isLowFidelitySurface(maybeSurface);
                setSurface(maybeSurface);
                setStatusText(`Rendered ${maybeSurface.surfaceId}`);
                setToolHitCount((count) => count + 1);
              }
              continue;
            }

            lastToolResponseSeen = toolResponse.response;
            appendEvent(`tool:${toolResponse.name}`, runtimeProtocol, toolResponse.response);
            const maybeSurface =
              extractSurfaceFromToolResponse(toolResponse.response) ??
              extractSurfaceFromRenderToolInvocation(toolResponse.name, toolResponse.response);
            if (maybeSurface) {
              renderDetected = true;
              latestSurface = maybeSurface;
              lowFidelityDetected = isLowFidelitySurface(maybeSurface);
              setSurface(maybeSurface);
              setStatusText(`Rendered ${maybeSurface.surfaceId}`);
            }
          }

          if (runtimeProtocol === 'ag_ui' && toolResponses.length === 0) {
            const eventSurface = extractSurfaceFromToolResponse({
              protocol: 'ag_ui',
              events: [runtimeEvent],
            });
            if (eventSurface) {
              renderDetected = true;
              latestSurface = eventSurface;
              lowFidelityDetected = isLowFidelitySurface(eventSurface);
              setSurface(eventSurface);
              setStatusText(`Rendered ${eventSurface.surfaceId}`);
            }
          }

          for (const artifactDelta of artifactDeltas) {
            const cacheKey = `${artifactDelta.name}@${artifactDelta.version}`;
            if (fetchedArtifactVersions.has(cacheKey)) {
              continue;
            }
            fetchedArtifactVersions.add(cacheKey);

            try {
              const artifactResponse = await fetch(
                `${baseUrl}/api/sessions/${selectedExample.id}/user1/${activeSessionId}/artifacts/${encodeURIComponent(artifactDelta.name)}`,
              );

              if (!artifactResponse.ok) {
                appendEvent(
                  `artifact:${artifactDelta.name}`,
                  runtimeProtocol,
                  { status: artifactResponse.status, version: artifactDelta.version },
                  `artifact fetch failed (${artifactResponse.status})`,
                );
                continue;
              }

              const artifactBody = await artifactResponse.text();
              appendEvent(
                `artifact:${artifactDelta.name}`,
                runtimeProtocol,
                artifactBody,
                `artifact version ${artifactDelta.version}`,
              );

              let rendered = false;
              const extractedSurface = extractSurfaceFromToolResponse(artifactBody);
              if (extractedSurface) {
                renderDetected = true;
                latestSurface = extractedSurface;
                lowFidelityDetected = isLowFidelitySurface(extractedSurface);
                setSurface(extractedSurface);
                setStatusText(`Rendered ${extractedSurface.surfaceId} from artifact`);
                rendered = true;
              }

              if (!rendered) {
                try {
                  const parsedArtifact = JSON.parse(artifactBody);
                  const maybeSurface = extractSurfaceFromToolResponse(parsedArtifact);
                  if (maybeSurface) {
                    renderDetected = true;
                    latestSurface = maybeSurface;
                    lowFidelityDetected = isLowFidelitySurface(maybeSurface);
                    setSurface(maybeSurface);
                    setStatusText(`Rendered ${maybeSurface.surfaceId} from artifact`);
                  }
                } catch {
                  // ignore non-JSON artifacts
                }
              }
            } catch (error) {
              appendEvent(
                `artifact:${artifactDelta.name}`,
                runtimeProtocol,
                { error: error instanceof Error ? error.message : 'artifact fetch failed' },
                'artifact fetch error',
              );
            }
          }

          if (isTerminalEvent) {
            // Some runtimes emit the model terminal event before trailing tool responses.
            // Keep draining the stream until EOF or an explicit [DONE] sentinel so response-side
            // render payloads can override speculative tool-call projection.
            continue;
          }
        }

        if (receivedDoneSentinel) {
          try {
            await reader.cancel();
          } catch {
            // ignore cancellation errors
          }
          break;
        }
      }

      const shouldRetry = options?.allowRetry !== false && !renderDetected && !isAutoRetry;
      if (shouldRetry) {
        setIsRetrying(true);
        setStatusText('Streaming UI (retrying)...');
        appendEvent(
          'retry',
          selectedProtocol,
          { reason: 'No renderable UI surface detected in streamed tool payloads.', attempt: 2 },
          'No renderable UI surface detected. Retrying once...',
        );
        return await runPrompt(rawPrompt, { retryAttempt: 1, activeSessionId, retryMode: 'no_surface' });
      }

      const shouldRetryQuality =
        options?.allowRetry !== false && renderDetected && lowFidelityDetected && !isAutoRetry;
      if (shouldRetryQuality) {
        setIsRetrying(true);
        setStatusText('Streaming UI (retrying quality)...');
        appendEvent(
          'retry',
          selectedProtocol,
          {
            reason: 'Low-fidelity UI detected (mostly text/buttons without structured components).',
            attempt: 2,
            surfaceId: latestSurface?.surfaceId ?? 'main',
          },
          'Low-fidelity UI detected. Retrying once with stricter structure constraints...',
        );
        return await runPrompt(rawPrompt, { retryAttempt: 1, activeSessionId, retryMode: 'quality' });
      }

      if (!assistantTextCaptured) {
        const fallback = buildAssistantFallback(
          lastRuntimeEventSeen,
          lastToolResponseSeen,
          renderDetected,
        );
        if (fallback) {
          setAssistantText(fallback);
        }
      }

      setRunCount((value) => value + 1);
      setStatusText('Completed');
      setLastLatencyMs(Math.round(performance.now() - startedAt));
    } catch (error) {
      const message = normalizeRequestError(error, baseUrl, endpoint, selectedExample.port);
      setStatusText('Failed');
      setErrorText(message);
      appendEvent('error', selectedProtocol, { message }, message);
    } finally {
      setIsStreaming(false);
      if (!isAutoRetry) {
        setIsRetrying(false);
      }
    }
  }

  useEffect(() => {
    agUiContextRef.current = { toolCallNames: new Map(), toolCallArgs: new Map() };
    mcpAppsBridgeRef.current = {};
    setSessionId(null);
    setSurface(null);
    setAssistantText('');
    setActivityMessages([]);
    setErrorText(null);
    setIsRetrying(false);
    setStatusText('Idle');
    setRunCount(0);
    setToolHitCount(0);
    setLastLatencyMs(null);
    setEventLog([]);
    void loadCapabilities();
  }, [baseUrl]);

  const surfaceSourceLabel = surface?.source ? formatCapabilityToken(surface.source) : null;
  const surfaceDisplayMode =
    typeof surface?.bridge?.hostContext?.displayMode === 'string'
      ? (surface.bridge.hostContext.displayMode as string)
      : null;
  const surfaceProtocolVersion =
    typeof surface?.bridge?.protocolVersion === 'string'
      ? surface.bridge.protocolVersion
      : null;
  const surfaceHostLabel =
    typeof surface?.bridge?.hostInfo?.title === 'string'
      ? (surface.bridge.hostInfo.title as string)
      : typeof surface?.bridge?.hostInfo?.name === 'string'
        ? (surface.bridge.hostInfo.name as string)
        : null;
  const surfaceBridgeState =
    surface?.bridge?.initialized === true
      ? 'initialized'
      : null;

  return (
    <div className="showcase-shell">
      <div className="ambient-orb ambient-orb-a" />
      <div className="ambient-orb ambient-orb-b" />
      <div className="ambient-grid" />

      <header className="hero-card fade-in-rise">
        <div>
          <p className="eyebrow">ADK-RUST UI EXAMPLES</p>
          <h1>ADK UI</h1>
          <p className="hero-subtitle">
            Create dynamic user interfaces with ADK UI, with support for standard protocols such
            as <strong>A2UI</strong>, <strong>AG-UI</strong>, and <strong>MCP Apps</strong>.
          </p>
        </div>

        <div className="hero-metrics">
          <div className="metric-card">
            <span>Runs</span>
            <strong>{runCount}</strong>
          </div>
          <div className="metric-card">
            <span>Tool Responses</span>
            <strong>{toolHitCount}</strong>
          </div>
          <div className="metric-card">
            <span>Last Latency</span>
            <strong>{lastLatencyMs ? `${lastLatencyMs}ms` : 'n/a'}</strong>
          </div>
          <div className="metric-card">
            <span>Status</span>
            <strong className={isStreaming ? 'status-live' : ''}>{statusText}</strong>
          </div>
        </div>
      </header>

      <section className="control-row fade-in-rise delay-1">
        <div className="select-group">
          <label htmlFor="example-select">Demo App</label>
          <select
            id="example-select"
            value={selectedExample.id}
            onChange={(event) => {
              const next = EXAMPLES.find((example) => example.id === event.target.value);
              if (next) {
                setSelectedExample(next);
              }
            }}
          >
            {EXAMPLES.map((example) => (
              <option key={example.id} value={example.id}>
                {example.name} ({example.port})
              </option>
            ))}
          </select>
          <p>{selectedExample.description}</p>
        </div>

        <div className="select-group">
          <label htmlFor="protocol-select">Protocol Profile</label>
          <select
            id="protocol-select"
            value={selectedProtocol}
            onChange={(event) => {
              const normalized = normalizeProtocol(event.target.value);
              if (normalized) {
                setSelectedProtocol(normalized);
              }
            }}
          >
            {PROTOCOLS.map((protocol) => (
              <option key={protocol.id} value={protocol.id}>
                {protocol.label}
              </option>
            ))}
          </select>
          <p>{selectedProtocolMeta.hint}</p>
        </div>

        <div className="capability-group">
          <div className="capability-title">
            <span>Server Capability Signal</span>
            <button type="button" onClick={() => void loadCapabilities()}>
              Refresh
            </button>
          </div>
          {capabilityForSelectedProtocol ? (
            <>
              <div className="chip-row">
                {capabilityForSelectedProtocol.versions.map((version) => (
                  <span className="chip" key={version}>
                    {version}
                  </span>
                ))}
                {capabilityForSelectedProtocol.implementationTier ? (
                  <span
                    className={`chip chip-${capabilityForSelectedProtocol.implementationTier}`}
                  >
                    Mode: {formatCapabilityToken(capabilityForSelectedProtocol.implementationTier)}
                  </span>
                ) : null}
                {capabilityForSelectedProtocol.specTrack ? (
                  <span className={`chip chip-${capabilityForSelectedProtocol.specTrack}`}>
                    Spec: {formatCapabilityToken(capabilityForSelectedProtocol.specTrack)}
                  </span>
                ) : null}
              </div>
              {capabilityForSelectedProtocol.summary ? (
                <p className="capability-summary">{capabilityForSelectedProtocol.summary}</p>
              ) : null}
              <div className="feature-list">
                {capabilityForSelectedProtocol.features.slice(0, 5).map((feature) => (
                  <span key={feature}>{feature}</span>
                ))}
              </div>
              {capabilityForSelectedProtocol.limitations?.length ? (
                <div className="capability-limitations">
                  {capabilityForSelectedProtocol.limitations.slice(0, 2).map((limitation) => (
                    <span key={limitation}>{limitation}</span>
                  ))}
                </div>
              ) : null}
              {capabilityForSelectedProtocol.deprecation ? (
                <div className="deprecation-note">
                  Legacy window: announced {capabilityForSelectedProtocol.deprecation.announcedOn}
                  {capabilityForSelectedProtocol.deprecation.sunsetTargetOn
                    ? `, sunset ${capabilityForSelectedProtocol.deprecation.sunsetTargetOn}`
                    : ''}
                </div>
              ) : null}
            </>
          ) : (
            <p className="capability-fallback">
              {capabilitiesError
                ? `Capability endpoint unavailable: ${capabilitiesError}`
                : 'No capability metadata loaded for selected protocol.'}
            </p>
          )}
        </div>
      </section>

      <section className="showcase-grid fade-in-rise delay-2">
        <aside className="panel prompt-panel">
          <div className="panel-header">
            <h2>Prompt Launcher</h2>
            <span>{selectedExample.port}</span>
          </div>

          <div className="prompt-grid">
            {selectedExample.prompts.map((prompt) => (
              <button
                key={prompt}
                type="button"
                className="prompt-button"
                onClick={() => void runPrompt(prompt)}
                disabled={isStreaming}
              >
                {prompt}
              </button>
            ))}
          </div>

          <div className="composer-block">
            <label htmlFor="prompt-input">Custom Prompt</label>
            <textarea
              id="prompt-input"
              value={promptInput}
              onChange={(event) => setPromptInput(event.target.value)}
              placeholder="Describe the UI you want the agent to render..."
              rows={5}
              disabled={isStreaming}
            />
            <button
              type="button"
              className="run-button"
              disabled={isStreaming || promptInput.trim().length === 0}
              onClick={() => {
                const value = promptInput;
                setPromptInput('');
                void runPrompt(value);
              }}
            >
              {isRetrying ? 'Retrying...' : isStreaming ? 'Streaming...' : 'Run Live Prompt'}
            </button>
          </div>
        </aside>

        <main className="panel canvas-panel">
          <div className="panel-header">
            <h2>Live Render Canvas</h2>
            <span>{surface ? surface.surfaceId : 'waiting'}</span>
          </div>

          {surfaceSourceLabel ? (
            <div className="surface-meta-strip">
              <span>Source: {surfaceSourceLabel}</span>
              {surfaceDisplayMode ? <span>Mode: {surfaceDisplayMode}</span> : null}
              {surfaceProtocolVersion ? <span>Protocol: {surfaceProtocolVersion}</span> : null}
              {surfaceHostLabel ? <span>Host: {surfaceHostLabel}</span> : null}
              {surfaceBridgeState ? <span>Bridge: {surfaceBridgeState}</span> : null}
            </div>
          ) : null}

          {surface ? (
            <div className="render-surface" data-surface={surface.surfaceId}>
              {surface.html ? (
                <iframe
                  srcDoc={surface.html}
                  title="AWP Rendered Surface"
                  sandbox="allow-scripts"
                  style={{
                    width: '100%',
                    minHeight: '500px',
                    border: 'none',
                    borderRadius: '8px',
                    background: 'white',
                  }}
                />
              ) : (
                surface.components.map((component, index) => (
                  <UiRenderer
                    key={`${component.id ?? 'component'}-${index}`}
                    component={component}
                    onAction={registerUiAction}
                  />
                ))
              )}
            </div>
          ) : (
            <div className="empty-state">
              {isStreaming ? (
                <h3 className="streaming-title" aria-live="polite">
                  {isRetrying ? 'Streaming UI (retrying)' : 'Streaming UI'}
                  <span className="streaming-dots" aria-hidden="true">
                    <span>.</span>
                    <span>.</span>
                    <span>.</span>
                  </span>
                </h3>
              ) : (
                <h3>No surface rendered yet</h3>
              )}
              {!isStreaming ? (
                <p>
                  Run a prompt to stream tool output and render a dynamic UI surface for{' '}
                  <strong>{selectedProtocolMeta.label}</strong>.
                </p>
              ) : null}
            </div>
          )}

          <div className="assistant-output">
            <h3>Assistant Text Output</h3>
            <pre>{assistantText || 'No text output captured yet.'}</pre>
          </div>

          {selectedProtocol === 'ag_ui' ? (
            <div className="activity-output">
              <h3>AG-UI Activity Snapshot</h3>
              {activityMessages.length === 0 ? (
                <p className="activity-empty">No activity messages captured yet.</p>
              ) : (
                <div className="activity-list">
                  {activityMessages.map((activity) => (
                    <article key={activity.messageId} className="activity-item">
                      <div className="activity-item-header">
                        <span className="activity-badge">{activity.activityType}</span>
                        <span className="activity-id">{activity.messageId}</span>
                      </div>
                      <pre>{JSON.stringify(activity.content, null, 2)}</pre>
                    </article>
                  ))}
                </div>
              )}
            </div>
          ) : null}
        </main>

        <aside className="panel stream-panel">
          <div className="panel-header">
            <h2>Runtime Event Stream</h2>
            <span>{eventLog.length}</span>
          </div>

          {errorText ? <div className="error-banner">Error: {errorText}</div> : null}

          <div className="event-list">
            {eventLog.length === 0 ? (
              <div className="event-empty">Events will appear here during streaming.</div>
            ) : (
              eventLog.map((entry) => (
                <details key={entry.id} className="event-item">
                  <summary>
                    <div>
                      <span className="event-kind">{entry.kind}</span>
                      <span className="event-preview">{entry.preview}</span>
                    </div>
                    <div className="event-meta">
                      <span>{entry.protocol}</span>
                      <span>{entry.at}</span>
                    </div>
                  </summary>
                  <pre>{JSON.stringify(entry.raw, null, 2)}</pre>
                </details>
              ))
            )}
          </div>
        </aside>
      </section>
    </div>
  );
}

export default App;
