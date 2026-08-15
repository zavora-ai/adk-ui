import React, { createContext, useContext } from 'react';
import type { Component } from './types';

export type UiKitStatus = 'draft' | 'approved' | 'deprecated';
export type UiKitAssetKind = 'logo' | 'icon' | 'illustration' | 'image' | 'font' | 'model_3d';
export type UiKitDensity = 'compact' | 'comfortable' | 'spacious';
export type UiKitRadius = 'none' | 'sm' | 'md' | 'lg' | 'xl';

export interface UiKitAsset {
    id: string;
    kind: UiKitAssetKind;
    uri: string;
    mimeType: string;
    alt: string;
    sha256?: string;
    license?: string;
    width?: number;
    height?: number;
    byteSize?: number;
}

export interface UiKitSemanticColors {
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
    input: string;
    ring: string;
    destructive: string;
    destructiveForeground: string;
    success: string;
    warning: string;
    info: string;
    chart: string[];
}

export interface UiKitManifest {
    schemaVersion: string;
    id: string;
    name: string;
    version: string;
    status: UiKitStatus;
    provenance?: {
        source: 'imported' | 'generated' | 'hybrid';
        sourceUri?: string;
        generatedBy?: string;
        reviewedBy?: string;
        approvedAt?: string;
    };
    brand: {
        vibe: string;
        industry?: string;
    };
    assets: UiKitAsset[];
    tokens: {
        colors: UiKitSemanticColors;
        typography: {
            bodyFamily: string;
            headingFamily: string;
            monoFamily: string;
            scale?: string;
        };
        density: UiKitDensity;
        radius: UiKitRadius;
        spacing: number;
        motionDuration: number;
    };
    components?: Record<string, {
        component: string;
        variant?: string;
        assetId?: string;
        slots?: Record<string, string>;
    }>;
    templates?: Array<{
        id: string;
        description?: string;
        requiredComponents?: string[];
    }>;
}

export interface UiKitSlotProps {
    component: Component;
    renderDefault: () => React.ReactNode;
}

export type UiKitSlots = Partial<Record<Component['type'], React.ComponentType<UiKitSlotProps>>>;

export const defaultUiKit: UiKitManifest = {
    schemaVersion: '1.0',
    id: 'zavora.ai:adk-ui/extended@0.2.0',
    name: 'ADK UI',
    version: '0.2.0',
    status: 'approved',
    provenance: {
        source: 'imported',
        reviewedBy: 'adk-ui maintainers',
        approvedAt: '2026-08-14',
    },
    brand: {
        vibe: 'clear, capable, calm',
        industry: 'agentic software',
    },
    assets: [],
    tokens: {
        colors: {
            primary: '#0f766e',
            primaryForeground: '#ffffff',
            secondary: '#e7f4f1',
            secondaryForeground: '#173f3a',
            background: '#f7f9f8',
            foreground: '#17211f',
            surface: '#ffffff',
            surfaceForeground: '#17211f',
            muted: '#edf2f1',
            mutedForeground: '#5b6b68',
            border: '#d8e1df',
            input: '#c7d3d0',
            ring: '#0f766e',
            destructive: '#b42318',
            destructiveForeground: '#ffffff',
            success: '#18794e',
            warning: '#a15c00',
            info: '#175cd3',
            chart: ['#0f766e', '#175cd3', '#a15c00', '#7a5af8', '#c4320a', '#0891b2'],
        },
        typography: {
            bodyFamily: 'Avenir Next, Avenir, sans-serif',
            headingFamily: 'Avenir Next, Avenir, sans-serif',
            monoFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
        },
        density: 'comfortable',
        radius: 'lg',
        spacing: 4,
        motionDuration: 160,
    },
};

interface UiKitRuntime {
    manifest: UiKitManifest;
    slots: UiKitSlots;
    provided: boolean;
}

const UiKitContext = createContext<UiKitRuntime>({
    manifest: defaultUiKit,
    slots: {},
    provided: false,
});

export function useAdkUiKit(): UiKitRuntime {
    return useContext(UiKitContext);
}

export interface AdkUiKitProviderProps {
    kit: UiKitManifest;
    slots?: UiKitSlots;
    /** Explicitly permits previewing a non-approved kit. Never enable this in production hosts. */
    allowDraft?: boolean;
    children: React.ReactNode;
}

export function AdkUiKitProvider({ kit, slots = {}, allowDraft = false, children }: AdkUiKitProviderProps) {
    if (kit.status !== 'approved' && !allowDraft) {
        throw new Error(`ADK UI kit "${kit.id}" is ${kit.status}; production rendering requires an approved kit.`);
    }

    return (
        <UiKitContext.Provider value={{ manifest: kit, slots, provided: true }}>
            <style data-adk-ui-kit-styles>{ADK_UI_KIT_CSS}</style>
            {children}
        </UiKitContext.Provider>
    );
}

export function resolveApprovedUiKit(
    requestedKitId: string | undefined,
    kits: readonly UiKitManifest[],
    fallbackKit: UiKitManifest = defaultUiKit,
): UiKitManifest {
    if (fallbackKit.status !== 'approved') {
        throw new Error('The fallback ADK UI kit must be approved.');
    }
    if (!requestedKitId) {
        return fallbackKit;
    }

    const requested = kits.find((kit) => kit.id === requestedKitId);
    return requested?.status === 'approved' ? requested : fallbackKit;
}

export function resolveKitAssetSource(kit: UiKitManifest, source: string): string | null {
    if (!source.startsWith('kit://')) {
        return source;
    }

    const assetId = source.slice('kit://'.length).trim();
    if (!assetId || assetId.includes('/') || assetId.includes('\\')) {
        return null;
    }

    return kit.assets.find((asset) => asset.id === assetId)?.uri ?? null;
}

export function kitStyleVariables(kit: UiKitManifest): React.CSSProperties & Record<string, string | number> {
    const { colors, typography, spacing, motionDuration } = kit.tokens;
    return {
        '--adk-ui-primary': colors.primary,
        '--adk-ui-primary-foreground': colors.primaryForeground,
        '--adk-ui-secondary': colors.secondary,
        '--adk-ui-secondary-foreground': colors.secondaryForeground,
        '--adk-ui-background': colors.background,
        '--adk-ui-foreground': colors.foreground,
        '--adk-ui-surface': colors.surface,
        '--adk-ui-surface-foreground': colors.surfaceForeground,
        '--adk-ui-muted': colors.muted,
        '--adk-ui-muted-foreground': colors.mutedForeground,
        '--adk-ui-border': colors.border,
        '--adk-ui-input': colors.input,
        '--adk-ui-ring': colors.ring,
        '--adk-ui-destructive': colors.destructive,
        '--adk-ui-destructive-foreground': colors.destructiveForeground,
        '--adk-ui-success': colors.success,
        '--adk-ui-warning': colors.warning,
        '--adk-ui-info': colors.info,
        '--adk-ui-font-body': typography.bodyFamily,
        '--adk-ui-font-heading': typography.headingFamily,
        '--adk-ui-font-mono': typography.monoFamily,
        '--adk-ui-radius': radiusValue(kit.tokens.radius),
        '--adk-ui-space': `${Math.max(1, spacing)}px`,
        '--adk-ui-motion-duration': `${Math.max(0, motionDuration)}ms`,
        ...Object.fromEntries(colors.chart.map((color, index) => [`--adk-ui-chart-${index + 1}`, color])),
    };
}

function radiusValue(radius: UiKitRadius): string {
    return {
        none: '0px',
        sm: '4px',
        md: '8px',
        lg: '12px',
        xl: '18px',
    }[radius];
}

export const ADK_UI_KIT_CSS = `
.adk-ui-kit-surface {
  color: var(--adk-ui-foreground);
  font-family: var(--adk-ui-font-body);
}
.adk-ui-kit-surface .adk-ui-heading {
  color: var(--adk-ui-surface-foreground);
  font-family: var(--adk-ui-font-heading);
}
.adk-ui-kit-surface .adk-ui-body,
.adk-ui-kit-surface .adk-ui-label { color: var(--adk-ui-foreground); }
.adk-ui-kit-surface .adk-ui-caption { color: var(--adk-ui-muted-foreground); }
.adk-ui-kit-surface .adk-ui-card,
.adk-ui-kit-surface .adk-ui-chart,
.adk-ui-kit-surface .adk-ui-modal {
  background: var(--adk-ui-surface);
  color: var(--adk-ui-surface-foreground);
  border-color: var(--adk-ui-border);
  border-radius: var(--adk-ui-radius);
}
.adk-ui-kit-surface .adk-ui-card__header,
.adk-ui-kit-surface .adk-ui-card__footer,
.adk-ui-kit-surface .adk-ui-table__head {
  background: var(--adk-ui-muted);
  border-color: var(--adk-ui-border);
}
.adk-ui-kit-surface .adk-ui-button {
  border-radius: var(--adk-ui-radius);
  transition-duration: var(--adk-ui-motion-duration);
}
.adk-ui-kit-surface .adk-ui-button--primary {
  background: var(--adk-ui-primary);
  color: var(--adk-ui-primary-foreground);
}
.adk-ui-kit-surface .adk-ui-button--secondary {
  background: var(--adk-ui-secondary);
  color: var(--adk-ui-secondary-foreground);
}
.adk-ui-kit-surface .adk-ui-button--danger {
  background: var(--adk-ui-destructive);
  color: var(--adk-ui-destructive-foreground);
}
.adk-ui-kit-surface .adk-ui-button--ghost { color: var(--adk-ui-foreground); }
.adk-ui-kit-surface .adk-ui-button--outline {
  color: var(--adk-ui-foreground);
  border-color: var(--adk-ui-border);
}
.adk-ui-kit-surface .adk-ui-control {
  background: var(--adk-ui-surface);
  color: var(--adk-ui-surface-foreground);
  border-color: var(--adk-ui-input);
  border-radius: var(--adk-ui-radius);
  accent-color: var(--adk-ui-primary);
}
.adk-ui-kit-surface .adk-ui-control:focus {
  border-color: var(--adk-ui-ring);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--adk-ui-ring) 20%, transparent);
}
.adk-ui-kit-surface .adk-ui-alert--info { color: var(--adk-ui-info); }
.adk-ui-kit-surface .adk-ui-alert--success { color: var(--adk-ui-success); }
.adk-ui-kit-surface .adk-ui-alert--warning { color: var(--adk-ui-warning); }
.adk-ui-kit-surface .adk-ui-alert--error { color: var(--adk-ui-destructive); }
.adk-ui-kit-surface .adk-ui-progress__track { background: var(--adk-ui-muted); }
.adk-ui-kit-surface .adk-ui-progress__value { background: var(--adk-ui-primary); }
.adk-ui-kit-surface .adk-ui-tabs { border-color: var(--adk-ui-border); }
.adk-ui-kit-surface .adk-ui-tab--active {
  border-color: var(--adk-ui-primary);
  color: var(--adk-ui-primary);
}
.adk-ui-kit-surface .adk-ui-table { border-color: var(--adk-ui-border); }
.adk-ui-kit-surface .adk-ui-table table,
.adk-ui-kit-surface .adk-ui-table tbody { background: var(--adk-ui-surface); }
.adk-ui-kit-surface .adk-ui-table th,
.adk-ui-kit-surface .adk-ui-table td { color: var(--adk-ui-surface-foreground); }
.adk-ui-kit-surface .adk-ui-divider { border-color: var(--adk-ui-border); }
.adk-ui-kit-surface .adk-ui-code { font-family: var(--adk-ui-font-mono); }
.adk-ui-scene-3d {
  position: relative;
  overflow: hidden;
  margin: 0;
  border: 1px solid var(--adk-ui-border);
  border-radius: calc(var(--adk-ui-radius) * 1.5);
  background: var(--adk-ui-surface);
  box-shadow: 0 24px 80px color-mix(in srgb, var(--adk-ui-foreground) 12%, transparent);
}
.adk-ui-scene-3d__viewport,
.adk-ui-scene-3d__viewport canvas { display: block; width: 100%; }
.adk-ui-scene-3d__caption {
  position: absolute;
  z-index: 1;
  top: 18px;
  left: 18px;
  display: grid;
  max-width: min(420px, calc(100% - 36px));
  gap: 3px;
  padding: 10px 13px;
  border: 1px solid color-mix(in srgb, var(--adk-ui-border) 72%, transparent);
  border-radius: var(--adk-ui-radius);
  background: color-mix(in srgb, var(--adk-ui-surface) 84%, transparent);
  backdrop-filter: blur(14px);
}
.adk-ui-scene-3d__caption strong { font-family: var(--adk-ui-font-heading); }
.adk-ui-scene-3d__caption span { color: var(--adk-ui-muted-foreground); font-size: 0.78rem; }
.adk-ui-scene-3d__actions {
  position: absolute;
  z-index: 1;
  right: 16px;
  bottom: 16px;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.adk-ui-scene-3d__actions button,
.adk-ui-scene-3d__actions span {
  border: 1px solid var(--adk-ui-border);
  border-radius: 999px;
  padding: 7px 10px;
  background: color-mix(in srgb, var(--adk-ui-surface) 88%, transparent);
  color: var(--adk-ui-foreground);
  font: 600 0.72rem var(--adk-ui-font-body);
  backdrop-filter: blur(12px);
}
.adk-ui-scene-3d__actions button { cursor: pointer; }
.adk-ui-scene-3d__loading,
.adk-ui-scene-3d__fallback {
  display: grid;
  place-items: center;
  min-height: 240px;
  border-radius: calc(var(--adk-ui-radius) * 1.5);
  background: linear-gradient(120deg, var(--adk-ui-muted), var(--adk-ui-surface), var(--adk-ui-muted));
  background-size: 220% 100%;
  color: var(--adk-ui-muted-foreground);
  animation: adk-ui-scene-loading 1.8s ease infinite;
}
@keyframes adk-ui-scene-loading { 50% { background-position: 100% 0; } }
@media (prefers-reduced-motion: reduce) { .adk-ui-scene-3d__loading { animation: none; } }
.adk-ui-optional-renderer {
  display: grid;
  gap: 0.25rem;
  place-content: center;
  padding: 1.5rem;
  min-height: 160px;
  text-align: center;
  border: 1px dashed var(--adk-ui-border);
  border-radius: calc(var(--adk-ui-radius) * 1.5);
  background: var(--adk-ui-muted);
  color: var(--adk-ui-muted-foreground);
}
.adk-ui-optional-renderer__title { font-weight: 600; }
.adk-ui-optional-renderer__hint { font-size: 0.875rem; }
.adk-ui-optional-renderer code {
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 0.8125rem;
}
.adk-ui-kit-surface .adk-ui-badge--default,
.adk-ui-kit-surface .adk-ui-badge--secondary {
  background: var(--adk-ui-muted);
  color: var(--adk-ui-muted-foreground);
}
`;
