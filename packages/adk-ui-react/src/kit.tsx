import { createContext, useContext } from 'react';
import type { CSSProperties, ReactNode } from 'react';

/** The parts of an adk-ui brand kit manifest a host applies. */
export interface KitManifest {
    id: string;
    status: 'draft' | 'approved' | 'deprecated';
    provenance?: { reviewedBy?: string; approvedAt?: string };
    tokens: {
        colors: {
            primary: string;
            background: string;
            surface: string;
            surfaceForeground: string;
            muted: string;
            mutedForeground: string;
            border: string;
            ring: string;
            chart?: string[];
        };
        typography: { bodyFamily: string; monoFamily: string };
        radius?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
    };
}

const HEX = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
const FUNCTIONAL = /^(rgb|rgba|hsl|hsla)\(\s*[\d.%\s,/+-]+\)$/;
const RADIUS = { none: '0px', sm: '4px', md: '8px', lg: '12px', xl: '18px' } as const;

/** Whether `value` is a CSS colour a kit may set: hex, or `rgb()` / `hsl()` of numbers. */
export function isKitColour(value: unknown): value is string {
    return typeof value === 'string' && (HEX.test(value.trim()) || FUNCTIONAL.test(value.trim()));
}

/** Whether a host may apply `kit`: approved, with its reviewer and approval date. */
export function isApprovedKit(kit: KitManifest): boolean {
    return kit.status === 'approved' && !!kit.provenance?.reviewedBy && !!kit.provenance?.approvedAt;
}

const quote = (family: string) => `"${family.replace(/["\\\n\r]/g, ' ')}"`;

/**
 * The adk-surface tokens `kit` sets, as CSS custom properties: card, border, text, accent,
 * focus, fonts, radius and the eight chart colours. Decision and status colours are never set,
 * and a value that is not a CSS colour is left out.
 */
export function kitStyleVariables(kit: KitManifest): Record<string, string> {
    const colors = kit.tokens.colors;
    const out: Record<string, string> = {};
    const colour = (name: string, value: unknown) => {
        if (isKitColour(value)) out[name] = value.trim();
    };
    colour('--bg', colors.background);
    colour('--card', colors.surface);
    colour('--raised', colors.muted);
    colour('--border', colors.border);
    colour('--text', colors.surfaceForeground);
    colour('--text-2', colors.mutedForeground);
    colour('--label', colors.mutedForeground);
    colour('--accent', colors.primary);
    colour('--focus', colors.ring);
    colour('--chart-grid', colors.muted);
    colour('--chart-axis', colors.mutedForeground);
    (colors.chart ?? []).slice(0, 8).forEach((value, index) => colour(`--chart-${index + 1}`, value));
    out['--font'] = `${quote(kit.tokens.typography.bodyFamily)}, ui-sans-serif, system-ui, sans-serif`;
    out['--mono'] = `${quote(kit.tokens.typography.monoFamily)}, ui-monospace, monospace`;
    out['--radius'] = RADIUS[kit.tokens.radius ?? 'md'];
    return out;
}

/**
 * The A2UI `theme` a host's transport adds to `createSurface` for a third-party renderer: the
 * primary colour alone, never an icon or a display name, and never part of a stored drawing.
 */
export function kitA2uiTheme(kit: KitManifest): Record<string, string> {
    return isKitColour(kit.tokens.colors.primary) ? { primaryColor: kit.tokens.colors.primary.trim() } : {};
}

const KitContext = createContext<KitManifest | null>(null);

/** The kit applied around this component, if any. */
export function useKit(): KitManifest | null {
    return useContext(KitContext);
}

/**
 * Applies an approved brand kit to everything inside it: `DrawnView` and adk-ui's renderers
 * read the tokens it sets. A draft or deprecated kit throws unless `allowDraft` is set, which
 * a production host never does. The ADK-Rust web home keeps its own tokens.
 */
export function KitProvider({ kit, allowDraft = false, children }: { kit: KitManifest; allowDraft?: boolean; children: ReactNode }) {
    if (!isApprovedKit(kit) && !allowDraft) {
        throw new Error(`adk-ui kit "${kit.id}" is ${kit.status} or unsigned; a host applies approved kits only.`);
    }
    return (
        <KitContext.Provider value={kit}>
            <div data-adk-ui-kit={kit.id} style={kitStyleVariables(kit) as CSSProperties}>
                {children}
            </div>
        </KitContext.Provider>
    );
}
