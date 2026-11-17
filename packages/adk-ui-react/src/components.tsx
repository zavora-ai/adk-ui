import { lazy, useCallback, useId, useState } from 'react';
import type { ReactNode } from 'react';
import { OptionalRenderer } from './OptionalRenderer';
import { formatNumber, formatValue, seriesColour } from './format';
import type {
    AppProps,
    BadgeProps,
    ChartProps,
    KeyValueProps,
    MetricProps,
    PageProps,
    ProgressProps,
    RegionProps,
    Row,
    Scene3DProps,
    SparklineProps,
} from './types';

const ChartBody = lazy(() => import('./lazy/ChartBody'));
const SceneBody = lazy(() => import('./lazy/SceneBody'));

/** What to draw while a rich renderer loads, or instead of it: the home's own view. */
interface Fallback {
    fallback?: ReactNode;
}

/**
 * A chart, drawn with `recharts` (an optional peer) in the host's `--chart-N` colours. Until
 * it loads, or where `recharts` is not installed, `fallback` is drawn, or a plain table.
 */
export function Chart({ fallback, ...chart }: ChartProps & Fallback) {
    const plain = fallback ?? <ChartTable chart={chart} />;
    return (
        <figure className="adk-ui-chart">
            {chart.title && <figcaption className="adk-ui-heading">{chart.title}</figcaption>}
            <OptionalRenderer peerDependency="recharts" capability="Chart" fallback={plain}>
                <ChartBody chart={chart} />
            </OptionalRenderer>
        </figure>
    );
}

/** A chart's rows as a table: its fallback without `recharts`. */
function ChartTable({ chart }: { chart: ChartProps }) {
    const rows = chart.window && chart.window < chart.data.length ? chart.data.slice(-chart.window) : chart.data;
    return (
        <table className="adk-ui-table">
            <thead>
                <tr>
                    <th scope="col">{chart.x.label ?? chart.x.key}</th>
                    {chart.series.map((series) => (
                        <th key={series.key} scope="col">
                            {series.label ?? series.key}
                        </th>
                    ))}
                </tr>
            </thead>
            <tbody>
                {rows.slice(0, 50).map((row, index) => (
                    <tr key={index}>
                        <td>{cellText(row[chart.x.key])}</td>
                        {chart.series.map((series) => (
                            <td key={series.key}>{formatValue(row[series.key], chart.valueFormat, chart.currency)}</td>
                        ))}
                    </tr>
                ))}
            </tbody>
        </table>
    );
}

function cellText(value: unknown): string {
    if (typeof value === 'number') return formatNumber(value);
    return value === null || value === undefined || typeof value === 'object' ? '' : String(value);
}

/** A small line of numbers, as an SVG; no peer needed. */
export function Sparkline({ data, key, label, tone, window }: SparklineProps & Fallback) {
    const numbers = (window && window < data.length ? data.slice(-window) : data)
        .map((entry) => (typeof entry === 'number' ? entry : Number((entry as Row)[key ?? 'value'])))
        .filter((n) => Number.isFinite(n));
    const low = Math.min(...numbers);
    const high = Math.max(...numbers);
    const points = numbers
        .map((n, i) => {
            const x = numbers.length > 1 ? (i / (numbers.length - 1)) * 116 + 2 : 60;
            const y = high > low ? 25 - ((n - low) / (high - low)) * 22 : 14;
            return `${x},${y}`;
        })
        .join(' ');
    const name = `${label ? `${label}: ` : ''}${numbers.length} values${numbers.length ? `, ${formatNumber(low)} to ${formatNumber(high)}` : ''}`;
    return (
        <span className="adk-ui-sparkline" data-tone={tone}>
            {label && <span className="adk-ui-caption">{label}</span>}
            <svg viewBox="0 0 120 28" role="img" aria-label={name}>
                <polyline points={points} fill="none" stroke={seriesColour(0)} strokeWidth={2} strokeLinejoin="round" />
            </svg>
        </span>
    );
}

const STATUS: Record<NonNullable<MetricProps['status']>, string | undefined> = {
    ok: 'OK',
    warning: 'Warning',
    critical: 'Critical',
    info: 'Info',
    neutral: undefined,
};

/** One number with its label, change, status and trend. */
export function Metric({ label, value, unit, format, currency, delta, caption, status, trend }: MetricProps & Fallback) {
    let change: string | undefined;
    let direction: 'better' | 'worse' | undefined;
    if (delta && typeof delta.value === 'number') {
        const sign = delta.value > 0 ? '+' : delta.value < 0 ? '-' : '';
        change = `${sign}${formatNumber(Math.abs(delta.value))}`;
        if (delta.goodWhen) {
            const up = delta.value > 0;
            const down = delta.value < 0;
            direction = (delta.goodWhen === 'up' && up) || (delta.goodWhen === 'down' && down) ? 'better' : up || down ? 'worse' : undefined;
        }
    }
    const word = status ? STATUS[status] : undefined;
    return (
        <div className="adk-ui-metric" data-status={status}>
            <span className="adk-ui-caption">{label}</span>
            <strong className="adk-ui-metric__value">{formatValue(value, format, currency, unit)}</strong>
            {change && (
                <span className="adk-ui-metric__delta" data-direction={direction}>
                    {change}
                    {direction && ` · ${direction}`}
                </span>
            )}
            {caption && <span className="adk-ui-caption">{caption}</span>}
            {word && <span className="adk-ui-metric__status">{word}</span>}
            {trend && trend.length > 1 && <Sparkline data={trend} label={undefined} />}
        </div>
    );
}

/** Labelled values, from a list or from rows of data. */
export function KeyValueList({ title, items, data, keyField, valueField }: KeyValueProps & Fallback) {
    const pairs = items
        ? items.map((item) => [cellText(item.key), formatValue(item.value)] as const)
        : (data ?? []).map((row) => [cellText(row[keyField ?? 'key']), formatValue(row[valueField ?? 'value'])] as const);
    return (
        <section className="adk-ui-key-value">
            {title && <h4 className="adk-ui-heading">{title}</h4>}
            <dl>
                {pairs.slice(0, 50).map(([key, value], index) => (
                    <div key={index} className="adk-ui-key-value__pair">
                        <dt>{key}</dt>
                        <dd>{value}</dd>
                    </div>
                ))}
            </dl>
        </section>
    );
}

/** How far a piece of work has come, out of 100. */
export function ProgressBar({ label, value, tone, indeterminate }: ProgressProps & Fallback) {
    const amount = Math.min(100, Math.max(0, value ?? 0));
    return (
        <div className="adk-ui-progress" data-tone={tone}>
            {label && <span className="adk-ui-progress__label">{label}</span>}
            <div
                className="adk-ui-progress__track"
                role="progressbar"
                aria-label={label}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={indeterminate ? undefined : amount}
                aria-valuetext={indeterminate ? 'in progress' : `${formatNumber(amount)}%`}
            >
                <div className="adk-ui-progress__bar" data-indeterminate={indeterminate || undefined} style={{ width: indeterminate ? '35%' : `${amount}%` }} />
            </div>
        </div>
    );
}

/** A short status word. */
export function Badge({ text, tone }: BadgeProps & Fallback) {
    return (
        <span className="adk-ui-badge" data-tone={tone ?? 'neutral'}>
            {text}
        </span>
    );
}

/** One page of an [`AppShell`]: its title and what it shows. */
export interface ShellPage {
    id: string;
    title: string;
    content: ReactNode;
}

/**
 * An application of pages with tabs between them. Switching pages is the host's own
 * navigation: it sends nothing to the agent.
 */
export function AppShell({ title, pages, navigation, initialPage }: Omit<AppProps, 'pages'> & { pages: ShellPage[] }) {
    const first = Math.max(0, pages.findIndex((page) => page.id === initialPage));
    const [active, setActive] = useState(first);
    const id = useId();
    return (
        <section className="adk-ui-app" data-navigation={navigation ?? 'top'}>
            {title && <h3 className="adk-ui-heading">{title}</h3>}
            <div className="adk-ui-app__tabs" role="tablist" aria-label={title ?? 'Pages'}>
                {pages.map((page, index) => (
                    <button
                        key={page.id}
                        type="button"
                        role="tab"
                        id={`${id}-tab-${index}`}
                        aria-selected={index === active}
                        aria-controls={`${id}-panel-${index}`}
                        tabIndex={index === active ? 0 : -1}
                        onClick={() => setActive(index)}
                    >
                        {page.title}
                    </button>
                ))}
            </div>
            {pages.map((page, index) => (
                <div
                    key={page.id}
                    role="tabpanel"
                    id={`${id}-panel-${index}`}
                    aria-labelledby={`${id}-tab-${index}`}
                    hidden={index !== active}
                    className="adk-ui-app__page"
                >
                    {page.content}
                </div>
            ))}
        </section>
    );
}

/** A page: a title, a description and what it holds. */
export function PageView({ title, description, layout, content }: Omit<PageProps, 'children'> & { content: ReactNode }) {
    return (
        <section className="adk-ui-page" data-layout={layout ?? 'stack'}>
            {title && <h3 className="adk-ui-heading">{title}</h3>}
            {description && <p className="adk-ui-caption">{description}</p>}
            <div className="adk-ui-page__content">{content}</div>
        </section>
    );
}

/** A titled region of a page, in a stack, a grid or a split. */
export function RegionView({ title, layout, columns, tone, variant, content }: Omit<RegionProps, 'children'> & { content: ReactNode }) {
    return (
        <section
            className="adk-ui-region"
            data-layout={layout ?? 'stack'}
            data-tone={tone}
            data-variant={variant ?? 'plain'}
            style={{ ['--adk-ui-columns' as string]: String(Math.min(4, Math.max(1, columns ?? 2))) }}
        >
            {title && <h4 className="adk-ui-heading">{title}</h4>}
            <div className="adk-ui-region__content">{content}</div>
        </section>
    );
}

/**
 * A scene of primitive shapes, drawn with `three` (an optional peer). Its labels are listed
 * beside it as text; nothing in it can be pressed. Until it loads, or without `three` or a
 * capable device, `fallback` is drawn, or the list alone.
 *
 * @deprecated For a host's own data only. Drawings in adk-ui's catalogs are drawn by
 * `DrawnView` itself, natively, from genui's checked scene; this component is frozen and goes in
 * 4.0.
 */
export function Scene3D({ fallback, ...scene }: Scene3DProps & Fallback) {
    const [unavailable, setUnavailable] = useState(false);
    const onUnavailable = useCallback(() => setUnavailable(true), []);
    const labels = scene.objects.filter((object) => object.label?.trim());
    const list = (
        <ul className="adk-ui-scene__labels" aria-label="What the scene holds">
            {labels.map((object) => (
                <li key={object.id}>{object.label}</li>
            ))}
        </ul>
    );
    return (
        <figure className="adk-ui-scene">
            {(scene.title || scene.description) && (
                <figcaption>
                    {scene.title && <strong className="adk-ui-heading">{scene.title}</strong>}
                    {scene.description && <span className="adk-ui-caption">{scene.description}</span>}
                </figcaption>
            )}
            {unavailable ? (
                (fallback ?? null)
            ) : (
                <OptionalRenderer peerDependency="three" capability="3D scene" fallback={fallback ?? null}>
                    <SceneBody scene={scene} onUnavailable={onUnavailable} />
                </OptionalRenderer>
            )}
            {labels.length > 0 && list}
        </figure>
    );
}
