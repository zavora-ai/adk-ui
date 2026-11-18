import {
    Area,
    AreaChart,
    Bar,
    BarChart,
    CartesianGrid,
    Legend,
    Line,
    LineChart,
    Pie,
    PieChart,
    ResponsiveContainer,
    Scatter,
    ScatterChart,
    Tooltip,
    XAxis,
    YAxis,
} from 'recharts';
import { compact, formatValue, seriesColour } from '../format';
import type { ChartProps, Row } from '../types';

/** A chart drawn with `recharts` in the host's chart colours. Loaded only when used. */
export default function ChartBody({ chart }: { chart: ChartProps }) {
    const rows = windowed(chart.data, chart.window);
    const xType = chart.x.type ?? 'category';
    const data = rows.map((row) => (xType === 'time' ? { ...row, [chart.x.key]: timeOf(row[chart.x.key]) } : row));
    const numericX = xType === 'number' || xType === 'time';
    const value = (n: unknown) => formatValue(n, chart.valueFormat, chart.currency);
    const tickX = xType === 'time' ? (n: number) => new Date(n).toLocaleString(undefined, timeParts(chart.timeUnit)) : undefined;
    const showLegend = chart.legend ?? chart.series.length > 1;
    const axes = (
        <>
            <CartesianGrid stroke="var(--chart-grid, #eef0f3)" strokeDasharray="3 3" />
            <XAxis
                dataKey={chart.x.key}
                type={numericX ? 'number' : 'category'}
                domain={numericX ? ['dataMin', 'dataMax'] : undefined}
                tickFormatter={tickX}
                tick={{ fill: 'var(--chart-axis, #64748b)', fontSize: 12 }}
                label={chart.x.label ? { value: chart.x.label, position: 'insideBottom', offset: -4 } : undefined}
            />
            <YAxis
                width={52}
                tickFormatter={(n: number) => compact(n)}
                tick={{ fill: 'var(--chart-axis, #64748b)', fontSize: 12 }}
                label={chart.yLabel ? { value: chart.yLabel, angle: -90, position: 'insideLeft' } : undefined}
            />
            <Tooltip formatter={(n) => value(n)} />
            {showLegend && <Legend />}
        </>
    );
    const name = (key: string) => chart.series.find((series) => series.key === key)?.label ?? key;
    const stack = chart.stacked ? 'stack' : undefined;
    return (
        <ResponsiveContainer width="100%" height={300}>
            {chart.kind === 'line' ? (
                <LineChart data={data}>
                    {axes}
                    {chart.series.map((series, i) => (
                        <Line key={series.key} type="monotone" dataKey={series.key} name={name(series.key)} stroke={seriesColour(i)} strokeWidth={2.5} dot={false} />
                    ))}
                </LineChart>
            ) : chart.kind === 'area' ? (
                <AreaChart data={data}>
                    {axes}
                    {chart.series.map((series, i) => (
                        <Area key={series.key} type="monotone" dataKey={series.key} name={name(series.key)} stackId={stack} stroke={seriesColour(i)} fill={seriesColour(i)} fillOpacity={0.18} strokeWidth={2.5} />
                    ))}
                </AreaChart>
            ) : chart.kind === 'pie' ? (
                <PieChart>
                    <Pie
                        data={data.map((row, i) => ({ ...row, fill: seriesColour(i) }))}
                        dataKey={chart.series[0]?.key ?? ''}
                        nameKey={chart.x.key}
                        innerRadius={54}
                        outerRadius={100}
                        paddingAngle={2}
                    />
                    <Tooltip formatter={(n) => value(n)} />
                    {showLegend && <Legend />}
                </PieChart>
            ) : chart.kind === 'scatter' ? (
                <ScatterChart>
                    <CartesianGrid stroke="var(--chart-grid, #eef0f3)" strokeDasharray="3 3" />
                    <XAxis dataKey="__x" type="number" domain={['dataMin', 'dataMax']} tickFormatter={tickX} />
                    <YAxis dataKey="__y" type="number" tickFormatter={(n: number) => compact(n)} />
                    <Tooltip formatter={(n) => value(n)} />
                    {chart.series.map((series, i) => (
                        <Scatter
                            key={series.key}
                            name={name(series.key)}
                            data={data.map((row) => ({ __x: row[chart.x.key], __y: row[series.key] }))}
                            fill={seriesColour(i)}
                        />
                    ))}
                    {showLegend && <Legend />}
                </ScatterChart>
            ) : (
                <BarChart data={data}>
                    {axes}
                    {chart.series.map((series, i) => (
                        <Bar key={series.key} dataKey={series.key} name={name(series.key)} stackId={stack} fill={seriesColour(i)} radius={[4, 4, 0, 0]} />
                    ))}
                </BarChart>
            )}
        </ResponsiveContainer>
    );
}

function windowed(rows: Row[], window?: number): Row[] {
    return window && window > 0 && window < rows.length ? rows.slice(-window) : rows;
}

function timeOf(value: unknown): number {
    if (typeof value === 'number') return value;
    const parsed = Date.parse(String(value));
    return Number.isFinite(parsed) ? parsed : 0;
}

function timeParts(unit: ChartProps['timeUnit']): Intl.DateTimeFormatOptions {
    switch (unit) {
        case 'second':
            return { hour: '2-digit', minute: '2-digit', second: '2-digit' };
        case 'minute':
        case 'hour':
            return { hour: '2-digit', minute: '2-digit' };
        case 'month':
            return { month: 'short', year: 'numeric' };
        case 'year':
            return { year: 'numeric' };
        default:
            return { month: 'short', day: 'numeric' };
    }
}
