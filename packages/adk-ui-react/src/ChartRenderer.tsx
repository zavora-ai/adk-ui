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
    Tooltip,
    XAxis,
    YAxis,
} from 'recharts';
import type { Component } from './types';
import { useAdkUiKit } from './kit';

type ChartComponent = Extract<Component, { type: 'chart' }>;

export default function ChartRenderer({ component }: { component: ChartComponent }) {
    const { manifest } = useAdkUiKit();
    const chartColors = component.colors?.length ? component.colors : manifest.tokens.colors.chart;
    const showLegend = component.show_legend !== false;
    const common = (
        <>
            <CartesianGrid strokeDasharray="3 3" stroke="#d9e0dd" />
            <XAxis dataKey={component.x_key} tick={{ fontSize: 12 }} />
            <YAxis tick={{ fontSize: 12 }} width={48} />
            <Tooltip />
            {showLegend && <Legend />}
        </>
    );

    return (
        <div className="adk-ui-chart mb-4 p-4 bg-white dark:bg-gray-900 border dark:border-gray-700 rounded-xl shadow-sm">
            {component.title && <h4 className="adk-ui-heading font-semibold text-lg mb-4 dark:text-white">{component.title}</h4>}
            <ResponsiveContainer width="100%" height={300}>
                {component.kind === 'line' ? (
                    <LineChart data={component.data}>
                        {common}
                        {component.y_keys.map((key, index) => (
                            <Line key={key} type="monotone" dataKey={key} stroke={chartColors[index % chartColors.length]} strokeWidth={2.5} dot={false} />
                        ))}
                    </LineChart>
                ) : component.kind === 'area' ? (
                    <AreaChart data={component.data}>
                        {common}
                        {component.y_keys.map((key, index) => (
                            <Area key={key} type="monotone" dataKey={key} fill={chartColors[index % chartColors.length]} fillOpacity={0.18} stroke={chartColors[index % chartColors.length]} strokeWidth={2.5} />
                        ))}
                    </AreaChart>
                ) : component.kind === 'pie' ? (
                    <PieChart>
                        <Pie data={component.data} dataKey={component.y_keys[0]} nameKey={component.x_key} cx="50%" cy="50%" outerRadius={100} innerRadius={54} paddingAngle={2}>
                            {component.data.map((_, index) => <Cell key={index} fill={chartColors[index % chartColors.length]} />)}
                        </Pie>
                        <Tooltip />
                        {showLegend && <Legend />}
                    </PieChart>
                ) : (
                    <BarChart data={component.data}>
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
