import {
    Area,
    AreaChart,
    Bar,
    BarChart,
    CartesianGrid,
    Cell,
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
import type { Component } from './types';
import { useAdkUiKit } from './kit';

type ChartComponent = Extract<Component, { type: 'chart' }>;
type ResolvedChartComponent = ChartComponent & { data: Record<string, unknown>[] };

export default function ChartRenderer({ component }: { component: ResolvedChartComponent }) {
    const { manifest } = useAdkUiKit();
    const chartColors = component.colors?.length ? component.colors : manifest.tokens.colors.chart;
    const showLegend = component.show_legend !== false;
    const windowSize = component.window && component.window > 0 ? component.window : component.data.length;
    const visibleData = component.data.slice(-windowSize).map((row) => component.x_type === 'time'
        ? { ...row, [component.x_key]: new Date(String(row[component.x_key])).getTime() }
        : row);
    const xAxisType = component.x_type === 'numeric' || component.x_type === 'time' ? 'number' : 'category';
    const tickFormatter = component.x_type === 'time'
        ? (value: number) => formatTime(value, component.time_format)
        : undefined;
    const common = (
        <>
            <CartesianGrid strokeDasharray="3 3" stroke="#d9e0dd" />
            <XAxis dataKey={component.x_key} type={xAxisType} domain={xAxisType === 'number' ? ['dataMin', 'dataMax'] : undefined} tick={{ fontSize: 12 }} tickFormatter={tickFormatter} label={component.x_label ? { value: component.x_label, position: 'insideBottom', offset: -4 } : undefined} />
            <YAxis tick={{ fontSize: 12 }} width={48} label={component.y_label ? { value: component.y_label, angle: -90, position: 'insideLeft' } : undefined} />
            <Tooltip />
            {showLegend && <Legend />}
        </>
    );

    return (
        <div className="adk-ui-chart mb-4 p-4 bg-white dark:bg-gray-900 border dark:border-gray-700 rounded-xl shadow-sm">
            {component.title && <h4 className="adk-ui-heading font-semibold text-lg mb-4 dark:text-white">{component.title}</h4>}
            <ResponsiveContainer width="100%" height={component.kind === 'sparkline' ? 96 : 300}>
                {component.kind === 'line' ? (
                    <LineChart data={visibleData}>
                        {common}
                        {component.y_keys.map((key, index) => (
                            <Line key={key} type="monotone" dataKey={key} stroke={chartColors[index % chartColors.length]} strokeWidth={2.5} dot={false} />
                        ))}
                    </LineChart>
                ) : component.kind === 'area' ? (
                    <AreaChart data={visibleData}>
                        {common}
                        {component.y_keys.map((key, index) => (
                            <Area key={key} type="monotone" dataKey={key} fill={chartColors[index % chartColors.length]} fillOpacity={0.18} stroke={chartColors[index % chartColors.length]} strokeWidth={2.5} />
                        ))}
                    </AreaChart>
                ) : component.kind === 'pie' ? (
                    <PieChart>
                        <Pie data={visibleData} dataKey={component.y_keys[0]} nameKey={component.x_key} cx="50%" cy="50%" outerRadius={100} innerRadius={54} paddingAngle={2}>
                            {visibleData.map((_, index) => <Cell key={index} fill={chartColors[index % chartColors.length]} />)}
                        </Pie>
                        <Tooltip />
                        {showLegend && <Legend />}
                    </PieChart>
                ) : component.kind === 'scatter' ? (
                    <ScatterChart>
                        <CartesianGrid strokeDasharray="3 3" stroke="#d9e0dd" />
                        <XAxis dataKey="__x" type="number" domain={['dataMin', 'dataMax']} tickFormatter={tickFormatter} label={component.x_label ? { value: component.x_label, position: 'insideBottom', offset: -4 } : undefined} />
                        <YAxis dataKey="__y" type="number" label={component.y_label ? { value: component.y_label, angle: -90, position: 'insideLeft' } : undefined} />
                        <Tooltip cursor={{ strokeDasharray: '3 3' }} />
                        {component.y_keys.map((key, index) => (
                            <Scatter
                                key={key}
                                name={key}
                                data={visibleData.map((row) => ({ __x: row[component.x_key], __y: row[key] }))}
                                fill={chartColors[index % chartColors.length]}
                            />
                        ))}
                        {showLegend && <Legend />}
                    </ScatterChart>
                ) : component.kind === 'sparkline' ? (
                    <LineChart data={visibleData} margin={{ top: 8, right: 4, bottom: 8, left: 4 }}>
                        <Tooltip />
                        {component.y_keys.map((key, index) => (
                            <Line key={key} type="monotone" dataKey={key} stroke={chartColors[index % chartColors.length]} strokeWidth={2.5} dot={false} />
                        ))}
                    </LineChart>
                ) : (
                    <BarChart data={visibleData}>
                        {common}
                        {component.y_keys.map((key, index) => (
                            <Bar key={key} dataKey={key} fill={chartColors[index % chartColors.length]} radius={[5, 5, 0, 0]} />
                        ))}
                    </BarChart>
                )}
            </ResponsiveContainer>
        </div>
    );
}

function formatTime(value: number, format?: string): string {
    const date = new Date(value);
    if (!Number.isFinite(date.getTime())) return String(value);
    if (format === 'date') return date.toLocaleDateString();
    if (format === 'time') return date.toLocaleTimeString();
    if (format === 'iso') return date.toISOString();
    return date.toLocaleString();
}
