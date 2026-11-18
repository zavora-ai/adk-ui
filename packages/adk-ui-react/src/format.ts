import type { ValueFormat } from './types';

/** `value` with thousands separators and at most two decimals, as the server writes it. */
export function formatNumber(value: number): string {
    if (!Number.isFinite(value)) return String(value);
    const rounded = Math.round(value * 100) / 100;
    const [whole, fraction] = Math.abs(rounded).toFixed(2).split('.');
    const grouped = whole.replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    const trimmed = fraction.replace(/0+$/, '');
    return `${rounded < 0 ? '-' : ''}${grouped}${trimmed ? `.${trimmed}` : ''}`;
}

/** A value in `format`, then `unit`: `4,200`, `42%`, `USD 4,200`, `120 ms`. */
export function formatValue(value: unknown, format?: ValueFormat, currency?: string, unit?: string): string {
    let core: string;
    if (typeof value === 'number') {
        if (format === 'percent') core = `${formatNumber(value)}%`;
        else if (format === 'currency' && currency) core = `${currency} ${formatNumber(value)}`;
        else core = formatNumber(value);
    } else if (value === null || value === undefined) {
        core = '';
    } else if (typeof value === 'object') {
        core = '';
    } else {
        core = String(value);
    }
    const suffix = unit?.trim();
    return suffix && core ? `${core} ${suffix}` : core;
}

/** A number as an axis reads it: `950`, `12.5k`, `3.4M`. */
export function compact(value: number): string {
    const abs = Math.abs(value);
    const trim = (n: number) => String(Number(n.toFixed(1)));
    if (abs >= 1e9) return `${trim(value / 1e9)}B`;
    if (abs >= 1e6) return `${trim(value / 1e6)}M`;
    if (abs >= 1e4) return `${trim(value / 1e3)}k`;
    return String(Number(value.toFixed(2)));
}

/** The CSS colour of series `index`: the host's `--chart-N` token. */
export const seriesColour = (index: number) => `var(--chart-${(index % 8) + 1}, #2563eb)`;
