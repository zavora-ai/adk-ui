import { defaultUiKit, type UiKitManifest } from '@zavora-ai/adk-ui-react';

type ShowcasePalette = {
  primary: string;
  primaryForeground: string;
  secondary: string;
  secondaryForeground: string;
  background: string;
  foreground: string;
  surface: string;
  surfaceForeground: string;
  muted: string;
  mutedForeground: string;
  border: string;
  ring: string;
  destructive?: string;
  success?: string;
  warning?: string;
  info?: string;
  chart: string[];
};

function createShowcaseKit(
  id: string,
  name: string,
  vibe: string,
  industry: string,
  palette: ShowcasePalette,
  options: {
    headingFamily?: string;
    bodyFamily?: string;
    density?: UiKitManifest['tokens']['density'];
    radius?: UiKitManifest['tokens']['radius'];
  } = {},
): UiKitManifest {
  return {
    ...defaultUiKit,
    id: `zavora.ai:adk-ui/showcase/${id}@1.0.0`,
    name,
    version: '1.0.0',
    brand: { vibe, industry },
    provenance: {
      source: 'generated',
      generatedBy: 'ADK UI showcase',
      reviewedBy: 'adk-ui maintainers',
      approvedAt: '2026-08-14',
    },
    tokens: {
      ...defaultUiKit.tokens,
      colors: {
        primary: palette.primary,
        primaryForeground: palette.primaryForeground,
        secondary: palette.secondary,
        secondaryForeground: palette.secondaryForeground,
        background: palette.background,
        foreground: palette.foreground,
        surface: palette.surface,
        surfaceForeground: palette.surfaceForeground,
        muted: palette.muted,
        mutedForeground: palette.mutedForeground,
        border: palette.border,
        input: palette.border,
        ring: palette.ring,
        destructive: palette.destructive ?? '#b42318',
        destructiveForeground: '#ffffff',
        success: palette.success ?? '#18794e',
        warning: palette.warning ?? '#a15c00',
        info: palette.info ?? '#175cd3',
        chart: palette.chart,
      },
      typography: {
        bodyFamily: options.bodyFamily ?? "'Avenir Next', 'Century Gothic', sans-serif",
        headingFamily: options.headingFamily ?? "'Avenir Next', 'Century Gothic', sans-serif",
        monoFamily: "'IBM Plex Mono', ui-monospace, monospace",
      },
      density: options.density ?? 'comfortable',
      radius: options.radius ?? 'lg',
    },
  };
}

export const incidentKit = createShowcaseKit(
  'northstar-ops',
  'Northstar Operations',
  'decisive, technical, high-signal',
  'site reliability',
  {
    primary: '#ff5a36', primaryForeground: '#111a1e', secondary: '#d8e8e2', secondaryForeground: '#17372f',
    background: '#f1eee7', foreground: '#111a1e', surface: '#fffdf8', surfaceForeground: '#111a1e',
    muted: '#e2e8e4', mutedForeground: '#56636a', border: '#cfd5d1', ring: '#ff5a36',
    chart: ['#ff5a36', '#08765e', '#3667d6', '#df9f28', '#8c5fb5', '#5c6b73'],
  },
  { density: 'compact', radius: 'md' },
);

export const supportKit = createShowcaseKit(
  'harbor-care',
  'Harbor Support',
  'calm, reassuring, precise',
  'customer support',
  {
    primary: '#0a6b58', primaryForeground: '#ffffff', secondary: '#f2d7bd', secondaryForeground: '#58331e',
    background: '#f6f3eb', foreground: '#18312b', surface: '#fffdf8', surfaceForeground: '#18312b',
    muted: '#e5eee9', mutedForeground: '#577068', border: '#cbdcd5', ring: '#0a6b58',
    chart: ['#0a6b58', '#d67842', '#2c64b7', '#84a94f', '#bf5b66', '#7c6dbe'],
  },
  { headingFamily: "'Bodoni 72', Didot, serif", radius: 'xl' },
);

export const releaseKit = createShowcaseKit(
  'signal-release',
  'Signal Release Room',
  'editorial, energetic, accountable',
  'software delivery',
  {
    primary: '#d9472b', primaryForeground: '#fffaf0', secondary: '#f6dc78', secondaryForeground: '#3d2d08',
    background: '#fff8e9', foreground: '#22201c', surface: '#fffdf7', surfaceForeground: '#22201c',
    muted: '#eee7d8', mutedForeground: '#686054', border: '#d9cfbc', ring: '#d9472b',
    chart: ['#d9472b', '#1d7062', '#4567bd', '#d39a29', '#6e5bb0', '#668455'],
  },
  { headingFamily: "'Bodoni 72', Didot, serif", radius: 'sm' },
);

export const commerceKit = createShowcaseKit(
  'atelier-commerce',
  'Atelier Commerce',
  'tactile, editorial, considered',
  'premium commerce',
  {
    primary: '#234f3f', primaryForeground: '#fffdf5', secondary: '#d8c39f', secondaryForeground: '#3b2d1f',
    background: '#f2ede2', foreground: '#20251f', surface: '#fffaf0', surfaceForeground: '#20251f',
    muted: '#e4ddcf', mutedForeground: '#68675f', border: '#d2c7b5', ring: '#234f3f',
    chart: ['#234f3f', '#b97a4d', '#8f9f75', '#405f82', '#a55449', '#c39d48'],
  },
  { headingFamily: "'Bodoni 72', Didot, serif", density: 'spacious', radius: 'none' },
);

export const spatialKit = createShowcaseKit(
  'orbit-logistics',
  'Orbit Logistics',
  'spatial, dark, exacting',
  'warehouse automation',
  {
    primary: '#b8ff68', primaryForeground: '#07130f', secondary: '#24463a', secondaryForeground: '#e5fff3',
    background: '#07130f', foreground: '#e8f5ef', surface: '#10241d', surfaceForeground: '#e8f5ef',
    muted: '#183128', mutedForeground: '#9bb8ac', border: '#2c493e', ring: '#b8ff68',
    destructive: '#ff6b62', success: '#65e6a7', warning: '#ffc85b', info: '#70b7ff',
    chart: ['#b8ff68', '#70b7ff', '#ffc85b', '#ff7f6b', '#c69cff', '#65e6a7'],
  },
  { density: 'compact', radius: 'md' },
);

export const streamingKit = createShowcaseKit(
  'pulse-stream',
  'Pulse Stream',
  'live, kinetic, observable',
  'financial operations',
  {
    primary: '#2859d6', primaryForeground: '#ffffff', secondary: '#dce5ff', secondaryForeground: '#18336f',
    background: '#f4f7fb', foreground: '#172033', surface: '#ffffff', surfaceForeground: '#172033',
    muted: '#e8edf5', mutedForeground: '#5b667b', border: '#d2dbe9', ring: '#2859d6',
    chart: ['#2859d6', '#12a17c', '#ee8b2b', '#d64f6a', '#7357c7', '#4d89a8'],
  },
  { radius: 'lg' },
);
