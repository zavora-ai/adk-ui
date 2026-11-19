//! `VegaLite`: a chart written as a Vega-Lite spec, the grammar BI tools and models already
//! know, drawn as the home's own chart. Only a subset is read — one mark over inline or bound
//! rows, with `x`, `y`, `color` (a series per value) and `theta` (a pie) — and anything else is
//! refused by name, so the agent knows what to change: no transforms, layers, facets, params,
//! selections or data from a URL.

use std::collections::BTreeMap;

use adk_surface::genui::{
    ChartKind as HomeChartKind, ExtensionComponent, HomeNode, ResolvedComponent,
};
use serde_json::{Map, Value, json};

use super::components::{ChartInput, MAX_CHART_ROWS, MAX_CHART_SERIES, chart_node};
use crate::schema::{component, data, text};

/// The top-level keys of a Vega-Lite spec this subset reads.
const KEYS: &[&str] = &[
    "$schema",
    "mark",
    "encoding",
    "data",
    "title",
    "description",
    "width",
    "height",
];

/// What Vega-Lite can say that this subset refuses, and why, by key.
const REFUSED: &[(&str, &str)] = &[
    (
        "transform",
        "transforms: aggregate or filter the rows before drawing",
    ),
    ("layer", "layers: draw one mark per chart"),
    ("facet", "facets: draw one chart per facet"),
    ("concat", "concatenation: draw one chart per view"),
    ("hconcat", "concatenation: draw one chart per view"),
    ("vconcat", "concatenation: draw one chart per view"),
    ("repeat", "repeat: draw one chart per field"),
    ("params", "params: a chart takes no input"),
    ("selection", "selections: a chart takes no input"),
];

/// A Vega-Lite spec, drawn as the home's chart.
pub(crate) struct VegaLiteComponent;

impl ExtensionComponent for VegaLiteComponent {
    fn name(&self) -> &'static str {
        "VegaLite"
    }

    fn schema(&self) -> Value {
        component(
            "VegaLite",
            json!({
                "title": text(),
                "spec": { "type": "object" },
                "data": data(json!({ "type": "object" }), MAX_CHART_ROWS)
            }),
            &["spec"],
        )
    }

    fn usage(&self) -> &'static str {
        "VegaLite: {\"component\": \"VegaLite\", \"title\", \"spec\": a Vega-Lite spec with one \
         mark (bar, line, area, point or arc) and encoding x, y, color (a series per value) or \
         theta (a pie), \"data\": {\"path\": \"/rows\"} or the spec's data.values (at most 500 \
         rows); no transforms, layers, facets or URLs."
    }

    fn home(&self, chart: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let spec = chart.get("spec").ok_or("it has no spec")?;
        let bound = chart
            .get("data")
            .and_then(Value::as_array)
            .map(Vec::as_slice);
        let read = read_spec(spec, bound)?;
        let title = chart.str("title").or(read.title.as_deref());
        chart_node(&ChartInput {
            kind: read.kind,
            rows: &read.rows,
            x_key: &read.x_key,
            x_type: read.x_type,
            x_label: read.x_label.as_deref(),
            series: read.series,
            y_label: read.y_label.as_deref(),
            title,
            time_unit: None,
        })
    }
}

/// A Vega-Lite spec read into what a chart draws.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReadSpec {
    pub(crate) kind: HomeChartKind,
    /// Wide rows: one per x value, a column per series.
    pub(crate) rows: Vec<Value>,
    pub(crate) x_key: String,
    pub(crate) x_type: &'static str,
    pub(crate) x_label: Option<String>,
    pub(crate) series: Vec<(String, String)>,
    pub(crate) y_label: Option<String>,
    pub(crate) title: Option<String>,
}

fn field<'a>(
    channel: Option<&'a Value>,
    name: &str,
) -> Result<Option<(&'a str, &'a str, Option<&'a str>)>, String> {
    let Some(channel) = channel else {
        return Ok(None);
    };
    for refused in ["aggregate", "bin", "timeUnit", "condition", "value"] {
        if channel.get(refused).is_some() {
            return Err(format!(
                "encoding.{name}.{refused} is not drawn here: put the computed values in the rows"
            ));
        }
    }
    let key = channel
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("encoding.{name} needs a field"))?;
    let kind = channel
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("nominal");
    let title = channel.get("title").and_then(Value::as_str);
    Ok(Some((key, kind, title)))
}

/// Reads `spec` (and `bound` rows, which win over `data.values`) into what a chart draws.
pub(crate) fn read_spec(spec: &Value, bound: Option<&[Value]>) -> Result<ReadSpec, String> {
    let object = spec.as_object().ok_or("a Vega-Lite spec is an object")?;
    for (key, why) in REFUSED {
        if object.contains_key(*key) {
            return Err(format!("a VegaLite chart draws no {why}"));
        }
    }
    if let Some(unknown) = object.keys().find(|key| !KEYS.contains(&key.as_str())) {
        return Err(format!(
            "{unknown} is not read here; use mark, encoding, data and title"
        ));
    }
    let mark = match object.get("mark") {
        Some(Value::String(mark)) => mark.as_str(),
        Some(Value::Object(mark)) => mark.get("type").and_then(Value::as_str).unwrap_or_default(),
        _ => return Err("a Vega-Lite spec needs a mark".to_string()),
    };
    let kind = match mark {
        "bar" => HomeChartKind::Bar,
        "line" => HomeChartKind::Line,
        "area" => HomeChartKind::Area,
        "point" | "circle" | "square" => HomeChartKind::Scatter,
        "arc" => HomeChartKind::Pie,
        other => {
            return Err(format!(
                "mark {other:?} is not drawn here; use bar, line, area, point or arc"
            ));
        }
    };
    let data = object.get("data");
    if data.and_then(|data| data.get("url")).is_some() {
        return Err("a VegaLite chart reads no data from a URL; bind the rows".to_string());
    }
    let rows: Vec<Value> = match bound {
        Some(rows) => rows.to_vec(),
        None => data
            .and_then(|data| data.get("values"))
            .and_then(Value::as_array)
            .cloned()
            .ok_or("a VegaLite chart needs rows: data.values or a bound data")?,
    };
    if rows.len() > MAX_CHART_ROWS {
        return Err(format!(
            "it has {} rows; a chart draws at most {MAX_CHART_ROWS}",
            rows.len()
        ));
    }
    let encoding = object
        .get("encoding")
        .and_then(Value::as_object)
        .ok_or("a Vega-Lite spec needs an encoding")?;
    let channel = |name: &str| encoding.get(name);
    let title = object.get("title").and_then(|title| match title {
        Value::String(text) => Some(text.clone()),
        Value::Object(title) => title
            .get("text")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    });
    if kind == HomeChartKind::Pie {
        let (value, _, value_title) =
            field(channel("theta"), "theta")?.ok_or("an arc needs encoding.theta")?;
        let (label, _, _) = field(channel("color"), "color")?
            .ok_or("an arc needs encoding.color for its slices")?;
        return Ok(ReadSpec {
            kind,
            rows,
            x_key: label.to_string(),
            x_type: "category",
            x_label: None,
            series: vec![(value.to_string(), value_title.unwrap_or(value).to_string())],
            y_label: None,
            title,
        });
    }
    let (x_key, x_kind, x_title) = field(channel("x"), "x")?.ok_or("a chart needs encoding.x")?;
    let (y_key, _, y_title) = field(channel("y"), "y")?.ok_or("a chart needs encoding.y")?;
    let x_type = match x_kind {
        "quantitative" => "number",
        "temporal" => "time",
        _ => "category",
    };
    let (rows, series) = match field(channel("color"), "color")? {
        // Long rows, a series per color value: pivoted to one row per x value.
        Some((color, _, _)) => {
            let mut order: Vec<String> = Vec::new();
            let mut wide: BTreeMap<String, Map<String, Value>> = BTreeMap::new();
            let mut positions: Vec<String> = Vec::new();
            for row in &rows {
                let x = row.get(x_key).cloned().unwrap_or(Value::Null);
                let id = x.to_string();
                let group = match row.get(color) {
                    Some(Value::String(group)) => group.clone(),
                    Some(other) => other.to_string(),
                    None => return Err(format!("a row has no {color}")),
                };
                if !order.contains(&group) {
                    order.push(group.clone());
                }
                let entry = wide.entry(id.clone()).or_insert_with(|| {
                    positions.push(id.clone());
                    let mut entry = Map::new();
                    entry.insert(x_key.to_string(), x.clone());
                    entry
                });
                entry.insert(group, row.get(y_key).cloned().unwrap_or(Value::Null));
            }
            if order.len() > MAX_CHART_SERIES {
                return Err(format!(
                    "{color} has {} values; a chart draws at most {MAX_CHART_SERIES} series",
                    order.len()
                ));
            }
            let rows = positions
                .iter()
                .filter_map(|id| wide.remove(id))
                .map(|mut row| {
                    for group in &order {
                        row.entry(group.clone()).or_insert(json!(0));
                    }
                    Value::Object(row)
                })
                .collect();
            (
                rows,
                order
                    .into_iter()
                    .map(|group| (group.clone(), group))
                    .collect(),
            )
        }
        None => (
            rows,
            vec![(y_key.to_string(), y_title.unwrap_or(y_key).to_string())],
        ),
    };
    Ok(ReadSpec {
        kind,
        rows,
        x_key: x_key.to_string(),
        x_type,
        x_label: x_title.map(str::to_string),
        series,
        y_label: y_title.map(str::to_string),
        title,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_spec_reads_into_a_chart() {
        let spec = json!({
            "mark": "bar",
            "data": { "values": [{ "q": "Q1", "rev": 4 }, { "q": "Q2", "rev": 7 }] },
            "encoding": { "x": { "field": "q", "type": "nominal", "title": "Quarter" },
                          "y": { "field": "rev", "type": "quantitative", "title": "Revenue" } }
        });
        let read = read_spec(&spec, None).unwrap();
        assert_eq!(read.kind, HomeChartKind::Bar);
        assert_eq!(read.x_type, "category");
        assert_eq!(
            read.series,
            vec![("rev".to_string(), "Revenue".to_string())]
        );
        assert_eq!(read.x_label.as_deref(), Some("Quarter"));
    }

    #[test]
    fn long_rows_with_a_color_become_a_series_each() {
        let spec = json!({
            "mark": { "type": "line" },
            "encoding": { "x": { "field": "day", "type": "temporal" },
                          "y": { "field": "ms", "type": "quantitative" },
                          "color": { "field": "service", "type": "nominal" } }
        });
        let rows = vec![
            json!({ "day": "2026-11-01", "service": "api", "ms": 120 }),
            json!({ "day": "2026-11-01", "service": "web", "ms": 80 }),
            json!({ "day": "2026-11-02", "service": "api", "ms": 140 }),
        ];
        let read = read_spec(&spec, Some(&rows)).unwrap();
        assert_eq!(read.x_type, "time");
        assert_eq!(read.series.len(), 2);
        assert_eq!(read.rows.len(), 2);
        assert_eq!(
            read.rows[1]["web"],
            json!(0),
            "a missing value draws as zero"
        );
    }

    #[test]
    fn an_arc_is_a_pie_of_theta_by_color() {
        let spec = json!({
            "mark": "arc",
            "data": { "values": [{ "k": "a", "v": 1 }, { "k": "b", "v": 3 }] },
            "encoding": { "theta": { "field": "v", "type": "quantitative" },
                          "color": { "field": "k", "type": "nominal" } }
        });
        let read = read_spec(&spec, None).unwrap();
        assert_eq!(read.kind, HomeChartKind::Pie);
        assert_eq!(read.x_key, "k");
    }

    #[test]
    fn what_the_subset_does_not_draw_is_refused_by_name() {
        for (spec, says) in [
            (
                json!({ "mark": "bar", "transform": [], "encoding": {} }),
                "transforms",
            ),
            (json!({ "layer": [] }), "layers"),
            (
                json!({ "mark": "bar", "data": { "url": "https://x.test/d.csv" }, "encoding": {} }),
                "URL",
            ),
            (
                json!({ "mark": "rect", "data": { "values": [] }, "encoding": {} }),
                "not drawn here",
            ),
            (
                json!({ "mark": "bar", "data": { "values": [] },
                     "encoding": { "x": { "field": "a" }, "y": { "field": "b", "aggregate": "sum" } } }),
                "aggregate",
            ),
            (
                json!({ "mark": "bar", "autosize": "fit", "encoding": {} }),
                "autosize",
            ),
        ] {
            let error = read_spec(&spec, Some(&[])).unwrap_err();
            assert!(error.contains(says), "{spec}: {error}");
        }
    }
}
