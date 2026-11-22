import type { ReactNode } from 'react';

/**
 * A node of a drawing as `@zavora-ai/adk-surface-react`'s `DrawnView` hands it to a renderer:
 * the subset of its `DrawnNode` adk-ui reads. Every string in it was checked on the server.
 */
export interface DrawnNodeLike {
    key: string;
    component: string;
    children?: DrawnNodeLike[];
}

/**
 * What a renderer of an extension component gets from `DrawnView`
 * (`ExtensionRendererProps`): the node (its one child is what the ADK-Rust home shows for
 * it), the component's properties as the server resolved and checked them, the nodes it holds
 * and `renderChild` to draw one as the home does. There is no way to send or change a value:
 * an extension component only shows.
 */
export interface RendererProps {
    node: DrawnNodeLike;
    props: Record<string, unknown>;
    children: DrawnNodeLike[];
    renderChild: (node: DrawnNodeLike) => ReactNode;
}

/** A renderer of one extension component. */
export type Renderer = (props: RendererProps) => ReactNode;

/** Renderers by catalog id, then component type: `DrawnView`'s `renderers`. */
export type Renderers = Record<string, Record<string, Renderer>>;

/** A tone; the host draws each in its own colours, never a decision's. */
export type Tone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

/** How a value is written. */
export type ValueFormat = 'number' | 'percent' | 'currency';

/** A row of data: field names to values. */
export type Row = Record<string, unknown>;

/** The insight catalog's `Chart`. */
export interface ChartProps {
    kind: 'bar' | 'line' | 'area' | 'pie' | 'scatter';
    title?: string;
    data: Row[];
    x: { key: string; type?: 'category' | 'number' | 'time'; label?: string };
    series: { key: string; label?: string }[];
    yLabel?: string;
    stacked?: boolean;
    legend?: boolean;
    valueFormat?: ValueFormat;
    currency?: string;
    timeUnit?: 'second' | 'minute' | 'hour' | 'day' | 'month' | 'year';
    window?: number;
}

/** The insight catalog's `Sparkline`. */
export interface SparklineProps {
    data: (number | Row)[];
    key?: string;
    label?: string;
    tone?: Tone;
    window?: number;
}

/** The insight catalog's `Metric`. */
export interface MetricProps {
    label: string;
    value: unknown;
    unit?: string;
    format?: ValueFormat;
    currency?: string;
    delta?: { value: number; goodWhen?: 'up' | 'down' };
    caption?: string;
    status?: 'ok' | 'warning' | 'critical' | 'info' | 'neutral';
    trend?: number[];
}

/** The insight catalog's `KeyValue`. */
export interface KeyValueProps {
    title?: string;
    items?: { key: string; value: unknown }[];
    data?: Row[];
    keyField?: string;
    valueField?: string;
}

/** The insight catalog's `Progress`. */
export interface ProgressProps {
    label?: string;
    value?: number;
    tone?: Tone;
    indeterminate?: boolean;
}

/** The insight catalog's `Badge`. */
export interface BadgeProps {
    text: string;
    tone?: Tone;
}

/** The studio catalog's `App`. */
export interface AppProps {
    title?: string;
    pages: { id: string; title: string; child: string }[];
    navigation?: 'top' | 'side';
    initialPage?: string;
}

/** The studio catalog's `Page`. */
export interface PageProps {
    title?: string;
    description?: string;
    layout?: 'stack' | 'grid' | 'split';
    children: string[];
}

/** The studio catalog's `Region`. */
export interface RegionProps {
    title?: string;
    layout?: 'stack' | 'grid' | 'split';
    columns?: number;
    tone?: Tone;
    variant?: 'plain' | 'hero';
    children: string[];
}
