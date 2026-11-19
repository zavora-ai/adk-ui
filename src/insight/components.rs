//! The insight components as `genui` extension components: their schemas and what the home
//! and the terminal show for each. Every one only shows; `genui` checks every string in it
//! and in the data it binds before anything is drawn.

use std::sync::Arc;

use adk_surface::genui::{
    ChartAxis, ChartKind as HomeChartKind, ChartSeries, ChartTick, DrawnChart, ExtensionComponent,
    HomeNode, ResolvedComponent,
};
use serde_json::{Number, Value, json};

use super::format::{cell_number, cell_text, cell_time, number, shown, time_ticks};
use crate::schema::{TONES, component, data, key, number as number_schema, one_of, text, value};

/// The most rows a chart draws.
pub const MAX_CHART_ROWS: usize = 500;
/// The most series a chart draws.
pub const MAX_CHART_SERIES: usize = 8;
/// The most pairs a key-value list shows.
pub const MAX_PAIRS: usize = 50;

/// Every insight component, for [`Catalog::extend`](adk_surface::genui::Catalog::extend).
pub fn components() -> Vec<Arc<dyn ExtensionComponent>> {
    vec![
        Arc::new(ChartComponent),
        Arc::new(super::vega::VegaLiteComponent),
        Arc::new(SparklineComponent),
        Arc::new(MetricComponent),
        Arc::new(KeyValueComponent),
        Arc::new(ProgressComponent),
        Arc::new(BadgeComponent),
    ]
}

/// The rows a component binds or lists, cut to the last `window` when one is given.
fn rows<'a>(component: &'a ResolvedComponent<'_>, name: &str) -> Result<&'a [Value], String> {
    let rows = component
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name} is not a list"))?;
    let window = component
        .get("window")
        .and_then(Value::as_u64)
        .map(|w| w as usize);
    Ok(match window {
        Some(window) if window < rows.len() => &rows[rows.len() - window..],
        _ => rows,
    })
}

/// `node` under `title`, when there is one.
fn titled(title: Option<&str>, node: HomeNode) -> HomeNode {
    match title.map(str::trim).filter(|title| !title.is_empty()) {
        Some(title) => HomeNode::Column(vec![HomeNode::heading(4, title), node]),
        None => node,
    }
}

/// A chart of rows of data: bar, line, area, pie or scatter.
pub(crate) struct ChartComponent;

impl ExtensionComponent for ChartComponent {
    fn name(&self) -> &'static str {
        "Chart"
    }

    fn schema(&self) -> Value {
        let series = json!({
            "type": "object",
            "properties": { "key": key(), "label": text() },
            "required": ["key"],
            "additionalProperties": false
        });
        component(
            "Chart",
            json!({
                "kind": one_of(&["bar", "line", "area", "pie", "scatter"]),
                "title": text(),
                "data": data(json!({ "type": "object" }), MAX_CHART_ROWS),
                "x": {
                    "type": "object",
                    "properties": {
                        "key": key(),
                        "type": one_of(&["category", "number", "time"]),
                        "label": text()
                    },
                    "required": ["key"],
                    "additionalProperties": false
                },
                "series": {
                    "type": "array",
                    "items": series,
                    "minItems": 1,
                    "maxItems": MAX_CHART_SERIES
                },
                "yLabel": text(),
                "stacked": { "type": "boolean" },
                "legend": { "type": "boolean" },
                "valueFormat": one_of(&["number", "percent", "currency"]),
                "currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
                "timeUnit": one_of(&["second", "minute", "hour", "day", "month", "year"]),
                "window": { "type": "integer", "minimum": 1, "maximum": MAX_CHART_ROWS }
            }),
            &["kind", "data", "x", "series"],
        )
    }

    fn usage(&self) -> &'static str {
        "Chart: {\"component\": \"Chart\", \"kind\": \"bar|line|area|pie|scatter\", \"title\", \
         \"data\": {\"path\": \"/rows\"} (at most 500 row objects), \"x\": {\"key\", \"type\": \
         \"category|number|time\", \"label\"}, \"series\": [{\"key\", \"label\"}] (1 to 8; a pie \
         uses the first), \"yLabel\", \"valueFormat\": \"number|percent|currency\", \"currency\", \
         \"timeUnit\", \"window\"}."
    }

    fn home(&self, chart: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let kind = match chart.str("kind") {
            Some("bar") => HomeChartKind::Bar,
            Some("line") => HomeChartKind::Line,
            Some("area") => HomeChartKind::Area,
            Some("pie") => HomeChartKind::Pie,
            Some("scatter") => HomeChartKind::Scatter,
            other => return Err(format!("kind {other:?} is not a chart kind")),
        };
        let rows = rows(chart, "data")?;
        let x = chart.get("x").ok_or("it has no x axis")?;
        let x_key = x
            .get("key")
            .and_then(Value::as_str)
            .ok_or("its x axis has no key")?;
        let specs: Vec<(String, String)> = chart
            .get("series")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|series| {
                let key = series.get("key")?.as_str()?;
                let label = series.get("label").and_then(Value::as_str).unwrap_or(key);
                Some((key.to_string(), label.to_string()))
            })
            .collect();
        chart_node(&ChartInput {
            kind,
            rows,
            x_key,
            x_type: x.get("type").and_then(Value::as_str).unwrap_or("category"),
            x_label: x.get("label").and_then(Value::as_str),
            series: specs,
            y_label: chart.str("yLabel"),
            title: chart.str("title"),
            time_unit: chart.str("timeUnit"),
        })
    }
}

/// What a chart draws, however it was written: a `Chart`'s properties or a Vega-Lite spec.
pub(crate) struct ChartInput<'a> {
    pub(crate) kind: HomeChartKind,
    pub(crate) rows: &'a [Value],
    pub(crate) x_key: &'a str,
    pub(crate) x_type: &'a str,
    pub(crate) x_label: Option<&'a str>,
    /// Each series' key in the rows, and its label.
    pub(crate) series: Vec<(String, String)>,
    pub(crate) y_label: Option<&'a str>,
    pub(crate) title: Option<&'a str>,
    pub(crate) time_unit: Option<&'a str>,
}

/// The home's chart of `input`: every cell checked as a number, a time or a label.
pub(crate) fn chart_node(input: &ChartInput<'_>) -> Result<HomeNode, String> {
    let (kind, rows, x_key) = (input.kind, input.rows, input.x_key);
    let mut specs: Vec<(&str, String)> = input
        .series
        .iter()
        .map(|(key, label)| (key.as_str(), label.clone()))
        .collect();
    if kind == HomeChartKind::Pie {
        specs.truncate(1);
    }
    if specs.is_empty() {
        return Err("it has no series".to_string());
    }
    if specs.len() > MAX_CHART_SERIES {
        return Err(format!(
            "it has {} series; a chart draws at most {MAX_CHART_SERIES}",
            specs.len()
        ));
    }
    if rows.len() > MAX_CHART_ROWS {
        return Err(format!(
            "it has {} rows; a chart draws at most {MAX_CHART_ROWS}",
            rows.len()
        ));
    }
    let (axis, xs, ticks): (ChartAxis, Vec<Number>, Vec<ChartTick>) = match input.x_type {
        "number" => (
            ChartAxis::Number,
            rows.iter()
                .enumerate()
                .map(|(i, row)| cell_number(row.get(x_key), i, x_key))
                .collect::<Result<_, _>>()?,
            Vec::new(),
        ),
        "time" => {
            let times: Vec<i64> = rows
                .iter()
                .enumerate()
                .map(|(i, row)| cell_time(row.get(x_key), i, x_key))
                .collect::<Result<_, _>>()?;
            let ticks = match (times.iter().min(), times.iter().max()) {
                (Some(low), Some(high)) => time_ticks(*low, *high, input.time_unit)
                    .into_iter()
                    .map(|(at, text)| ChartTick {
                        at: Number::from(at),
                        text,
                    })
                    .collect(),
                _ => Vec::new(),
            };
            (
                ChartAxis::Time,
                times.into_iter().map(Number::from).collect(),
                ticks,
            )
        }
        _ => (
            ChartAxis::Category {
                labels: rows
                    .iter()
                    .enumerate()
                    .map(|(i, row)| cell_text(row.get(x_key), i, x_key))
                    .collect::<Result<_, _>>()?,
            },
            Vec::new(),
            Vec::new(),
        ),
    };
    let series = specs
        .into_iter()
        .map(|(key, name)| {
            let y = rows
                .iter()
                .enumerate()
                .map(|(i, row)| cell_number(row.get(key), i, key))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ChartSeries {
                name,
                x: xs.clone(),
                y,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let drawn = DrawnChart {
        kind,
        x: axis,
        x_label: input.x_label.map(str::to_string),
        y_label: input.y_label.map(str::to_string),
        ticks,
        series,
        summary: String::new(),
    };
    Ok(titled(input.title, HomeNode::Chart(drawn)))
}

/// A small line of numbers, no axes.
pub(crate) struct SparklineComponent;

impl ExtensionComponent for SparklineComponent {
    fn name(&self) -> &'static str {
        "Sparkline"
    }

    fn schema(&self) -> Value {
        component(
            "Sparkline",
            json!({
                "data": data(json!({ "type": ["number", "object"] }), MAX_CHART_ROWS),
                "key": key(),
                "label": text(),
                "tone": one_of(TONES),
                "window": { "type": "integer", "minimum": 1, "maximum": MAX_CHART_ROWS }
            }),
            &["data"],
        )
    }

    fn usage(&self) -> &'static str {
        "Sparkline: {\"component\": \"Sparkline\", \"data\": {\"path\": \"/values\"} (numbers, or \
         rows with \"key\"), \"label\", \"tone\", \"window\"}."
    }

    fn home(&self, sparkline: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let rows = rows(sparkline, "data")?;
        let key = sparkline.str("key");
        let y = rows
            .iter()
            .enumerate()
            .map(|(i, row)| match key {
                Some(key) => cell_number(row.get(key), i, key),
                None => cell_number(Some(row), i, "value"),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let label = sparkline.str("label").unwrap_or_default();
        let chart = HomeNode::Chart(DrawnChart {
            kind: HomeChartKind::Sparkline,
            x: ChartAxis::Number,
            x_label: None,
            y_label: None,
            ticks: Vec::new(),
            series: vec![ChartSeries {
                name: label.to_string(),
                x: (0..y.len()).map(Number::from).collect(),
                y,
            }],
            summary: String::new(),
        });
        Ok(if label.trim().is_empty() {
            chart
        } else {
            HomeNode::Column(vec![HomeNode::caption(label), chart])
        })
    }
}

/// One number with its label, change and trend.
pub(crate) struct MetricComponent;

impl ExtensionComponent for MetricComponent {
    fn name(&self) -> &'static str {
        "Metric"
    }

    fn schema(&self) -> Value {
        component(
            "Metric",
            json!({
                "label": text(),
                "value": value(),
                "unit": text(),
                "format": one_of(&["number", "percent", "currency"]),
                "currency": { "type": "string", "pattern": "^[A-Z]{3}$" },
                "delta": {
                    "type": "object",
                    "properties": {
                        "value": number_schema(),
                        "goodWhen": one_of(&["up", "down"])
                    },
                    "required": ["value"],
                    "additionalProperties": false
                },
                "caption": text(),
                "status": one_of(&["ok", "warning", "critical", "info", "neutral"]),
                "trend": data(json!({ "type": "number" }), MAX_CHART_ROWS)
            }),
            &["label", "value"],
        )
    }

    fn usage(&self) -> &'static str {
        "Metric: {\"component\": \"Metric\", \"label\", \"value\", \"unit\", \"format\": \
         \"number|percent|currency\", \"currency\", \"delta\": {\"value\", \"goodWhen\": \
         \"up|down\"}, \"caption\", \"status\": \"ok|warning|critical|info|neutral\", \"trend\": \
         {\"path\": \"/numbers\"}}."
    }

    fn home(&self, metric: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let label = metric.str("label").unwrap_or_default();
        let value = shown(
            metric.get("value").unwrap_or(&Value::Null),
            metric.str("format"),
            metric.str("currency"),
            metric.str("unit"),
        )?;
        let mut nodes = vec![HomeNode::caption(label), HomeNode::heading(2, value)];
        if let Some(delta) = metric.get("delta") {
            let change = delta
                .get("value")
                .and_then(Value::as_f64)
                .ok_or("its delta's value is not a number")?;
            let sign = if change > 0.0 {
                "+"
            } else if change < 0.0 {
                "-"
            } else {
                ""
            };
            let mut line = format!("{sign}{}", number(change.abs()));
            let good = delta.get("goodWhen").and_then(Value::as_str);
            let better = matches!(good, Some("up")) && change > 0.0
                || matches!(good, Some("down")) && change < 0.0;
            let worse = matches!(good, Some("up")) && change < 0.0
                || matches!(good, Some("down")) && change > 0.0;
            if better {
                line.push_str(" · better");
            } else if worse {
                line.push_str(" · worse");
            }
            nodes.push(HomeNode::text(line));
        }
        if let Some(caption) = metric
            .str("caption")
            .filter(|caption| !caption.trim().is_empty())
        {
            nodes.push(HomeNode::caption(caption));
        }
        let status = match metric.str("status") {
            Some("ok") => Some("OK"),
            Some("warning") => Some("Warning"),
            Some("critical") => Some("Critical"),
            Some("info") => Some("Info"),
            _ => None,
        };
        if let Some(status) = status {
            nodes.push(HomeNode::caption(status));
        }
        if let Some(trend) = metric.get("trend") {
            let y = trend
                .as_array()
                .ok_or("its trend is not a list of numbers")?
                .iter()
                .enumerate()
                .map(|(i, value)| cell_number(Some(value), i, "trend"))
                .collect::<Result<Vec<_>, _>>()?;
            nodes.push(HomeNode::Chart(DrawnChart {
                kind: HomeChartKind::Sparkline,
                x: ChartAxis::Number,
                x_label: None,
                y_label: None,
                ticks: Vec::new(),
                series: vec![ChartSeries {
                    name: label.to_string(),
                    x: (0..y.len()).map(Number::from).collect(),
                    y,
                }],
                summary: String::new(),
            }));
        }
        Ok(HomeNode::Card(nodes))
    }
}

/// Labelled values, from a literal list or from rows of data.
pub(crate) struct KeyValueComponent;

impl ExtensionComponent for KeyValueComponent {
    fn name(&self) -> &'static str {
        "KeyValue"
    }

    fn schema(&self) -> Value {
        component(
            "KeyValue",
            json!({
                "title": text(),
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": { "key": text(), "value": value() },
                        "required": ["key", "value"],
                        "additionalProperties": false
                    },
                    "maxItems": MAX_PAIRS
                },
                "data": data(json!({ "type": "object" }), MAX_PAIRS),
                "keyField": key(),
                "valueField": key()
            }),
            &[],
        )
    }

    fn usage(&self) -> &'static str {
        "KeyValue: {\"component\": \"KeyValue\", \"title\", \"items\": [{\"key\", \"value\"}] (at \
         most 50), or \"data\": {\"path\": \"/rows\"} with \"keyField\" and \"valueField\"}."
    }

    fn home(&self, list: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let pairs: Vec<(String, String)> =
            if let Some(items) = list.get("items").and_then(Value::as_array) {
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        Ok((
                            cell_text(item.get("key"), i, "key")?,
                            shown(item.get("value").unwrap_or(&Value::Null), None, None, None)?,
                        ))
                    })
                    .collect::<Result<_, String>>()?
            } else if let Some(rows) = list.get("data").and_then(Value::as_array) {
                let key = list.str("keyField").ok_or("data needs a keyField")?;
                let value = list.str("valueField").ok_or("data needs a valueField")?;
                rows.iter()
                    .enumerate()
                    .map(|(i, row)| {
                        Ok((
                            cell_text(row.get(key), i, key)?,
                            shown(row.get(value).unwrap_or(&Value::Null), None, None, None)?,
                        ))
                    })
                    .collect::<Result<_, String>>()?
            } else {
                return Err("it has neither items nor data".to_string());
            };
        if pairs.len() > MAX_PAIRS {
            return Err(format!("it has {} pairs; at most {MAX_PAIRS}", pairs.len()));
        }
        let rows = pairs
            .into_iter()
            .map(|(key, value)| HomeNode::Row(vec![HomeNode::caption(key), HomeNode::text(value)]))
            .collect();
        Ok(titled(list.str("title"), HomeNode::Column(rows)))
    }
}

/// How far a piece of work has come, out of 100.
pub(crate) struct ProgressComponent;

impl ExtensionComponent for ProgressComponent {
    fn name(&self) -> &'static str {
        "Progress"
    }

    fn schema(&self) -> Value {
        component(
            "Progress",
            json!({
                "label": text(),
                "value": number_schema(),
                "tone": one_of(TONES),
                "indeterminate": { "type": "boolean" }
            }),
            &[],
        )
    }

    fn usage(&self) -> &'static str {
        "Progress: {\"component\": \"Progress\", \"label\", \"value\": 0 to 100, \"tone\", \
         \"indeterminate\"}. Move it with patch_surface as the work moves."
    }

    fn home(&self, progress: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let label = progress.str("label").unwrap_or_default().trim();
        let indeterminate = progress
            .get("indeterminate")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let amount = if indeterminate {
            "in progress".to_string()
        } else {
            let value = progress
                .get("value")
                .and_then(Value::as_f64)
                .ok_or("its value is not a number")?;
            format!("{}%", number(value.clamp(0.0, 100.0)))
        };
        Ok(HomeNode::text(if label.is_empty() {
            amount
        } else {
            format!("{label} · {amount}")
        }))
    }
}

/// A short status word.
pub(crate) struct BadgeComponent;

impl ExtensionComponent for BadgeComponent {
    fn name(&self) -> &'static str {
        "Badge"
    }

    fn schema(&self) -> Value {
        component(
            "Badge",
            json!({ "text": text(), "tone": one_of(TONES) }),
            &["text"],
        )
    }

    fn usage(&self) -> &'static str {
        "Badge: {\"component\": \"Badge\", \"text\", \"tone\": \"neutral|info|success|warning|\
         danger\"}; a status word such as \"Shipped\", never a decision's word."
    }

    fn home(&self, badge: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        Ok(HomeNode::caption(badge.str("text").unwrap_or_default()))
    }
}
