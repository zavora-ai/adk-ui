//! The spatial components as `genui` extension components. Each one only shows: the home draws
//! the scenes natively from `genui`'s [`DrawnScene`] and the cards and layouts with CSS 3D.
//! Picking an object fills a choice field the drawing shows and sends nothing. Data bound with
//! `{"path"}` skips the schema, so every limit is checked again in `home`.

use std::collections::BTreeMap;
use std::sync::Arc;

use adk_surface::genui::{
    Axis, CameraPreset, ChildSlot, DepthEffect, DrawnDepth, DrawnScene, ExtensionComponent,
    HomeNode, HomePanel, HomeScene, LabelSize, LightKind, MotionKind, ResolvedComponent, SceneAxes,
    SceneCamera, SceneColor, SceneFinish, SceneKind, SceneLight, SceneLighting, SceneLook,
    SceneMotion, SceneObject, SceneSelect, SceneShape, SceneValues,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::scene::{
    COLORS, EXTENT, MAX_LABELS, STATES, arc, cell, color, finite, flat, framed, label,
    layered_layout, line, num, on_globe, point, range, scale, short, sphere_layout, state,
    text_view, ticks, titled, unit_shape, xyz,
};
use crate::schema::{component, component_id, data, key, one_of, text, vector};

/// The most rows a 3D bar chart draws.
pub const MAX_BAR_ROWS: usize = 50;
/// The most series a 3D chart draws: one per chart colour.
pub const MAX_SERIES: usize = 8;
/// The most points a 3D scatter chart draws.
pub const MAX_SCATTER_POINTS: usize = 2_000;
/// The most points along each side of a surface plot.
pub const MAX_SURFACE_SIDE: usize = 64;
/// The most nodes a 3D network draws.
pub const MAX_GRAPH_NODES: usize = 200;
/// The most links a 3D network draws.
pub const MAX_GRAPH_EDGES: usize = 400;
/// The most places a globe shows.
pub const MAX_GLOBE_POINTS: usize = 500;
/// The most routes a globe shows.
pub const MAX_GLOBE_ARCS: usize = 200;
/// The most objects a spatial scene has.
pub const MAX_OBJECTS: usize = 128;
/// The most objects of a scene that move.
pub const MAX_MOVING: usize = 32;
/// The most components a scene shows beside its objects.
pub const MAX_PANELS: usize = 4;
/// The most lights a scene adds.
pub const MAX_LIGHTS: usize = 3;
/// The most components a stage, wall or carousel presents.
pub const MAX_STAGED: usize = 12;
/// The most layers a depth card stacks behind its front.
pub const MAX_LAYERS: usize = 4;
/// The points a globe's route is drawn through.
const ARC_SEGMENTS: usize = 16;
/// The globe's radius.
const GLOBE: f64 = 4.0;

/// Every spatial component beyond the studio ones, for
/// [`Catalog::extend`](adk_surface::genui::Catalog::extend).
pub fn components() -> Vec<Arc<dyn ExtensionComponent>> {
    vec![
        Arc::new(Bar3DComponent),
        Arc::new(Scatter3DComponent),
        Arc::new(SurfacePlotComponent),
        Arc::new(Graph3DComponent),
        Arc::new(GlobeComponent),
        Arc::new(SceneComponent),
        Arc::new(DepthCardComponent),
        Arc::new(StageComponent),
    ]
}

/// The properties every 3D view shares, beside its own.
fn view(mut own: Value) -> Value {
    own["title"] = text();
    own["description"] = text();
    own["height"] = json!({ "type": "integer", "minimum": 240, "maximum": 960 });
    own["controls"] = one_of(&["none", "orbit"]);
    own
}

fn axis_spec() -> Value {
    json!({
        "type": "object",
        "properties": { "key": key(), "label": text() },
        "required": ["key"],
        "additionalProperties": false
    })
}

fn row() -> Value {
    json!({ "type": "object" })
}

/// The rows a component binds or holds, at most `max`.
fn rows<'a>(
    component: &'a ResolvedComponent<'_>,
    name: &str,
    max: usize,
) -> Result<&'a [Value], String> {
    let rows = component
        .get(name)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{name} is not a list"))?;
    if rows.len() > max {
        return Err(format!("{name} has {} entries; at most {max}", rows.len()));
    }
    Ok(rows)
}

/// The key and label of an axis property.
fn axis_of(
    component: &ResolvedComponent<'_>,
    name: &str,
) -> Result<(String, Option<String>), String> {
    let axis = component
        .get(name)
        .ok_or_else(|| format!("{name} is missing"))?;
    let key = axis
        .get("key")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{name} has no key"))?;
    let label = axis
        .get("label")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok((key.to_string(), label))
}

/// A scene of `kind` in a component's frame, under its title and description.
fn finish(
    component: &ResolvedComponent<'_>,
    mut scene: DrawnScene,
    panels: Vec<HomePanel>,
    words: Option<HomeNode>,
) -> HomeNode {
    framed(
        &mut scene,
        component.get("height").and_then(Value::as_u64),
        component.str("controls"),
    );
    let body = HomeNode::Scene(Box::new(HomeScene {
        scene,
        panels,
        text: words,
    }));
    titled(component.str("title"), component.str("description"), body)
}

fn preset(name: Option<&str>) -> CameraPreset {
    match name {
        Some("front") => CameraPreset::Front,
        Some("top") => CameraPreset::Top,
        Some("side") => CameraPreset::Side,
        Some("iso") => CameraPreset::Iso,
        _ => CameraPreset::Fit,
    }
}

/// Bars over two categories: rows along one axis, series along the other.
pub(crate) struct Bar3DComponent;

impl ExtensionComponent for Bar3DComponent {
    fn name(&self) -> &'static str {
        "Bar3D"
    }

    fn schema(&self) -> Value {
        component(
            "Bar3D",
            view(json!({
                "data": data(row(), MAX_BAR_ROWS),
                "x": axis_spec(),
                "series": {
                    "type": "array",
                    "items": axis_spec(),
                    "minItems": 1,
                    "maxItems": MAX_SERIES
                },
                "yLabel": text(),
                "camera": one_of(&["fit", "front", "top", "side", "iso"])
            })),
            &["data", "x", "series"],
        )
    }

    fn usage(&self) -> &'static str {
        "Bar3D: {\"component\": \"Bar3D\", \"title\", \"data\": {\"path\": \"/rows\"}, \"x\": {\"key\", \
         \"label\"}, \"series\": [{\"key\", \"label\"}] (1 to 8), \"yLabel\"}: a value over two \
         categories, rows along x and series in depth; at most 50 rows."
    }

    fn home(&self, bars: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let rows = rows(bars, "data", MAX_BAR_ROWS)?;
        let (x_key, x_label) = axis_of(bars, "x")?;
        let series: Vec<(String, String)> = bars
            .get("series")
            .and_then(Value::as_array)
            .ok_or("series is not a list")?
            .iter()
            .map(|series| {
                let key = cell(series.get("key"));
                let label = series
                    .get("label")
                    .and_then(Value::as_str)
                    .unwrap_or(&key)
                    .to_string();
                (key, label)
            })
            .collect();
        if series.is_empty() || series.len() > MAX_SERIES {
            return Err(format!("it has {} series; 1 to {MAX_SERIES}", series.len()));
        }
        let mut values = Vec::with_capacity(rows.len() * series.len());
        for (index, row) in rows.iter().enumerate() {
            for (key, _) in &series {
                values.push(finite(row.get(key), &format!("row {index}'s {key}"))?);
            }
        }
        let high = values
            .iter()
            .fold(0.0_f64, |high, value| high.max(value.abs()));
        let factor = if high > 0.0 {
            EXTENT * 0.75 / high
        } else {
            1.0
        };
        let spacing = (EXTENT / rows.len().max(series.len()).max(1) as f64).clamp(0.4, 1.4);
        let x0 = -(rows.len() as f64 - 1.0) * spacing / 2.0;
        let z0 = -(series.len() as f64 - 1.0) * spacing / 2.0;
        let mut at = Vec::new();
        let mut heights = Vec::new();
        let mut indices = Vec::new();
        for (r, _) in rows.iter().enumerate() {
            for (s, _) in series.iter().enumerate() {
                at.push(x0 + r as f64 * spacing);
                at.push(z0 + s as f64 * spacing);
                heights.push(values[r * series.len() + s] * factor);
                indices.push(u8::try_from(s).unwrap_or(7));
            }
        }
        let mut scene = DrawnScene::new(SceneKind::Bars);
        // A bar's height times this is its value: the summary and readouts speak the data.
        scene.values = Some(SceneValues {
            scale: num(1.0 / factor)?,
            offset: num(0.0)?,
        });
        scene.objects.push(SceneObject::new(
            "bars",
            SceneShape::Bars {
                at: at.into_iter().map(num).collect::<Result<_, _>>()?,
                heights: heights.into_iter().map(num).collect::<Result<_, _>>()?,
                width: num(spacing * 0.7)?,
                series: indices,
            },
        ));
        // The camera starts on the +x, +z side: categories read along the near edge.
        let front = -z0 + spacing * 0.9;
        for (r, row) in rows.iter().enumerate() {
            let text = cell(row.get(&x_key));
            if !text.trim().is_empty() && r < MAX_LABELS {
                scene.objects.push(label(
                    format!("x-{r}"),
                    &text,
                    [x0 + r as f64 * spacing, 0.0, front],
                    LabelSize::Small,
                )?);
            }
        }
        let side = x0 + rows.len() as f64 * spacing;
        for (s, (_, name)) in series.iter().enumerate() {
            let mut tag = label(
                format!("series-{s}"),
                name,
                [side, 0.0, z0 + s as f64 * spacing],
                LabelSize::Small,
            )?;
            tag.look.color = SceneColor::chart(u8::try_from(s).unwrap_or(7));
            scene.objects.push(tag);
        }
        scene.axes = Some(SceneAxes {
            x: x_label,
            y: bars.str("yLabel").map(str::to_string),
            z: None,
            ticks: ticks(Axis::Y, 0.0, high, EXTENT * 0.75)?
                .into_iter()
                .map(|mut tick| {
                    // Bars stand on the ground: ticks from 0 up.
                    tick.at =
                        num(tick.at.as_f64().unwrap_or(0.0) + EXTENT * 0.375).unwrap_or(tick.at);
                    tick
                })
                .collect(),
            floor: true,
        });
        scene.camera.preset = match bars.str("camera") {
            None => CameraPreset::Iso,
            name => preset(name),
        };
        let words = rows
            .iter()
            .enumerate()
            .map(|(r, row)| {
                let parts: Vec<String> = series
                    .iter()
                    .enumerate()
                    .map(|(s, (_, name))| {
                        format!(
                            "{name} {}",
                            crate::insight::format::number(values[r * series.len() + s])
                        )
                    })
                    .collect();
                format!("{}: {}", short(&cell(row.get(&x_key))), parts.join(", "))
            })
            .collect();
        Ok(finish(bars, scene, Vec::new(), Some(text_view(words))))
    }
}

/// Points over three measures.
pub(crate) struct Scatter3DComponent;

impl ExtensionComponent for Scatter3DComponent {
    fn name(&self) -> &'static str {
        "Scatter3D"
    }

    fn schema(&self) -> Value {
        component(
            "Scatter3D",
            view(json!({
                "data": data(row(), MAX_SCATTER_POINTS),
                "x": axis_spec(),
                "y": axis_spec(),
                "z": axis_spec(),
                "group": key(),
                "labelKey": key(),
                "size": { "type": "number", "minimum": 0.02, "maximum": 1 }
            })),
            &["data", "x", "y", "z"],
        )
    }

    fn usage(&self) -> &'static str {
        "Scatter3D: {\"component\": \"Scatter3D\", \"title\", \"data\": {\"path\"}, \"x\", \"y\", \
         \"z\": {\"key\", \"label\"}, \"group\": \"<key>\", \"labelKey\": \"<key>\"}: points over \
         three measures, coloured by group (at most 8); at most 2000 points, 24 labelled."
    }

    fn home(&self, scatter: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let rows = rows(scatter, "data", MAX_SCATTER_POINTS)?;
        let axes = [
            axis_of(scatter, "x")?,
            axis_of(scatter, "y")?,
            axis_of(scatter, "z")?,
        ];
        let mut points: Vec<[f64; 3]> = Vec::with_capacity(rows.len());
        for (index, row) in rows.iter().enumerate() {
            let mut point = [0.0; 3];
            for (axis, (key, _)) in axes.iter().enumerate() {
                point[axis] = finite(row.get(key), &format!("row {index}'s {key}"))?;
            }
            points.push(point);
        }
        let ranges: Vec<(f64, f64)> = (0..3)
            .map(|axis| range(points.iter().map(|p| p[axis])).unwrap_or((0.0, 0.0)))
            .collect();
        let placed: Vec<[f64; 3]> = points
            .iter()
            .map(|p| {
                [
                    scale(p[0], ranges[0].0, ranges[0].1, EXTENT),
                    scale(p[1], ranges[1].0, ranges[1].1, EXTENT),
                    scale(p[2], ranges[2].0, ranges[2].1, EXTENT),
                ]
            })
            .collect();
        let mut groups: BTreeMap<String, u8> = BTreeMap::new();
        let series: Vec<u8> = match scatter.str("group") {
            Some(key) => rows
                .iter()
                .map(|row| {
                    let name = cell(row.get(key));
                    let next = u8::try_from(groups.len() % MAX_SERIES).unwrap_or(7);
                    *groups.entry(name).or_insert(next)
                })
                .collect(),
            None => Vec::new(),
        };
        let mut scene = DrawnScene::new(SceneKind::Points);
        let size = scatter.get("size").and_then(Value::as_f64).unwrap_or(0.12);
        scene.objects.push(SceneObject::new(
            "points",
            SceneShape::Points {
                at: flat(&placed)?,
                size: num(size)?,
                series,
            },
        ));
        if let Some(key) = scatter.str("labelKey") {
            for (index, (row, at)) in rows.iter().zip(&placed).enumerate().take(MAX_LABELS) {
                let text = cell(row.get(key));
                if !text.trim().is_empty() {
                    scene.objects.push(label(
                        format!("label-{index}"),
                        &text,
                        [at[0], at[1] + size * 2.5, at[2]],
                        LabelSize::Small,
                    )?);
                }
            }
        }
        let mut all_ticks = Vec::new();
        for (axis, which) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            all_ticks.extend(ticks(which, ranges[axis].0, ranges[axis].1, EXTENT)?);
        }
        scene.axes = Some(SceneAxes {
            x: axes[0].1.clone().or_else(|| Some(axes[0].0.clone())),
            y: axes[1].1.clone().or_else(|| Some(axes[1].0.clone())),
            z: axes[2].1.clone().or_else(|| Some(axes[2].0.clone())),
            ticks: all_ticks,
            floor: true,
        });
        let name = |axis: usize| axes[axis].1.clone().unwrap_or_else(|| axes[axis].0.clone());
        let mut words = vec![format!("{} points", rows.len())];
        for (axis, (low, high)) in ranges.iter().enumerate() {
            words.push(format!(
                "{}: from {} to {}",
                short(&name(axis)),
                crate::insight::format::number(*low),
                crate::insight::format::number(*high)
            ));
        }
        if !groups.is_empty() {
            let names: Vec<&str> = groups.keys().map(String::as_str).collect();
            words.push(format!("Groups: {}", names.join(", ")));
        }
        Ok(finish(scatter, scene, Vec::new(), Some(text_view(words))))
    }
}

/// A value over a grid of rows and columns.
pub(crate) struct SurfacePlotComponent;

impl ExtensionComponent for SurfacePlotComponent {
    fn name(&self) -> &'static str {
        "SurfacePlot"
    }

    fn schema(&self) -> Value {
        let grid = json!({
            "type": "array",
            "items": { "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": MAX_SURFACE_SIDE },
            "minItems": 2,
            "maxItems": MAX_SURFACE_SIDE
        });
        component(
            "SurfacePlot",
            view(json!({
                "values": { "oneOf": [{ "$ref": format!("{}/DataBinding", crate::schema::COMMON) }, grid] },
                "xLabel": text(),
                "yLabel": text(),
                "zLabel": text(),
                "wireframe": { "type": "boolean" }
            })),
            &["values"],
        )
    }

    fn usage(&self) -> &'static str {
        "SurfacePlot: {\"component\": \"SurfacePlot\", \"title\", \"values\": {\"path\"} (rows of \
         numbers, 2 to 64 by 2 to 64), \"xLabel\", \"yLabel\" (the value), \"zLabel\", \
         \"wireframe\"}: a value over a grid, coloured by height."
    }

    fn home(&self, plot: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let grid = rows(plot, "values", MAX_SURFACE_SIDE)?;
        let columns = grid.first().and_then(Value::as_array).map_or(0, Vec::len);
        if grid.len() < 2 || !(2..=MAX_SURFACE_SIDE).contains(&columns) {
            return Err(format!(
                "values is {} by {columns}; 2 to {MAX_SURFACE_SIDE} each way",
                grid.len()
            ));
        }
        let mut heights = Vec::with_capacity(grid.len() * columns);
        for (r, row) in grid.iter().enumerate() {
            let row = row
                .as_array()
                .filter(|row| row.len() == columns)
                .ok_or_else(|| format!("row {r} of values does not have {columns} numbers"))?;
            for (c, value) in row.iter().enumerate() {
                heights.push(finite(Some(value), &format!("values[{r}][{c}]"))?);
            }
        }
        let (low, high) = range(heights.iter().copied()).unwrap_or((0.0, 0.0));
        let placed: Vec<f64> = heights
            .iter()
            .map(|value| scale(*value, low, high, EXTENT * 0.5))
            .collect();
        let mut scene = DrawnScene::new(SceneKind::Surface);
        let length = EXTENT * 0.5;
        if high > low {
            scene.values = Some(SceneValues {
                scale: num((high - low) / length)?,
                offset: num(low + (high - low) / 2.0)?,
            });
        }
        let mut land = SceneObject::new(
            "surface",
            SceneShape::Grid {
                columns: u16::try_from(columns).map_err(|_| "too many columns")?,
                rows: u16::try_from(grid.len()).map_err(|_| "too many rows")?,
                heights: placed.into_iter().map(num).collect::<Result<_, _>>()?,
                width: num(EXTENT)?,
                depth: num(EXTENT)?,
            },
        );
        land.look = SceneLook {
            color: SceneColor::Ramp,
            wireframe: plot.get("wireframe") == Some(&Value::Bool(true)),
            ..SceneLook::default()
        };
        scene.objects.push(land);
        scene.axes = Some(SceneAxes {
            x: plot.str("xLabel").map(str::to_string),
            y: plot.str("yLabel").map(str::to_string),
            z: plot.str("zLabel").map(str::to_string),
            ticks: ticks(Axis::Y, low, high, EXTENT * 0.5)?,
            floor: false,
        });
        scene.camera.preset = CameraPreset::Iso;
        let peak = heights
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(i, _)| i);
        let words = vec![
            format!(
                "{} rows by {columns} columns, from {} to {}",
                grid.len(),
                crate::insight::format::number(low),
                crate::insight::format::number(high)
            ),
            format!(
                "Highest at row {}, column {}",
                peak / columns + 1,
                peak % columns + 1
            ),
        ];
        Ok(finish(plot, scene, Vec::new(), Some(text_view(words))))
    }
}

/// Nodes and the links between them.
pub(crate) struct Graph3DComponent;

impl ExtensionComponent for Graph3DComponent {
    fn name(&self) -> &'static str {
        "Graph3D"
    }

    fn schema(&self) -> Value {
        let node = json!({
            "type": "object",
            "properties": {
                "id": key(), "label": text(), "group": key(),
                "x": { "type": "number" }, "y": { "type": "number" }, "z": { "type": "number" }
            },
            "required": ["id"],
            "additionalProperties": false
        });
        let edge = json!({
            "type": "object",
            "properties": { "from": key(), "to": key() },
            "required": ["from", "to"],
            "additionalProperties": false
        });
        component(
            "Graph3D",
            view(json!({
                "nodes": data(node, MAX_GRAPH_NODES),
                "edges": data(edge, MAX_GRAPH_EDGES),
                "layout": one_of(&["sphere", "layers", "given"]),
                "spin": { "type": "boolean" }
            })),
            &["nodes", "edges"],
        )
    }

    fn usage(&self) -> &'static str {
        "Graph3D: {\"component\": \"Graph3D\", \"title\", \"nodes\": [{\"id\", \"label\", \"group\", \
         \"x\", \"y\", \"z\"}] (at most 200), \"edges\": [{\"from\", \"to\"}] (at most 400), \
         \"layout\": \"sphere|layers|given\", \"spin\"}: a network; layers stack the groups."
    }

    fn home(&self, graph: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let nodes = rows(graph, "nodes", MAX_GRAPH_NODES)?;
        let edges = rows(graph, "edges", MAX_GRAPH_EDGES)?;
        if nodes.is_empty() {
            return Err("it has no nodes".into());
        }
        let mut index: BTreeMap<String, usize> = BTreeMap::new();
        for (at, node) in nodes.iter().enumerate() {
            let id = cell(node.get("id"));
            if id.is_empty() || index.insert(id.clone(), at).is_some() {
                return Err(format!("node {at} has a missing or repeated id"));
            }
        }
        let mut links = Vec::with_capacity(edges.len());
        let mut degree = vec![0usize; nodes.len()];
        for (at, edge) in edges.iter().enumerate() {
            let end = |name: &str| {
                let id = cell(edge.get(name));
                index
                    .get(&id)
                    .copied()
                    .ok_or_else(|| format!("edge {at} joins '{id}', which is not a node"))
            };
            let (from, to) = (end("from")?, end("to")?);
            degree[from] += 1;
            degree[to] += 1;
            links.push((from, to));
        }
        let mut groups: BTreeMap<String, usize> = BTreeMap::new();
        let group_of: Vec<usize> = nodes
            .iter()
            .map(|node| {
                let next = groups.len();
                *groups.entry(cell(node.get("group"))).or_insert(next)
            })
            .collect();
        let placed = match graph.str("layout") {
            Some("given") => {
                let given: Vec<[f64; 3]> = nodes
                    .iter()
                    .enumerate()
                    .map(|(at, node)| {
                        Ok([
                            finite(node.get("x"), &format!("node {at}'s x"))?,
                            finite(node.get("y"), &format!("node {at}'s y"))?,
                            finite(node.get("z"), &format!("node {at}'s z"))?,
                        ])
                    })
                    .collect::<Result<_, String>>()?;
                let ranges: Vec<(f64, f64)> = (0..3)
                    .map(|axis| range(given.iter().map(|p| p[axis])).unwrap_or((0.0, 0.0)))
                    .collect();
                let wide = ranges
                    .iter()
                    .map(|(low, high)| high - low)
                    .fold(0.0_f64, f64::max);
                given
                    .iter()
                    .map(|p| {
                        let fit = |axis: usize| {
                            let middle = (ranges[axis].0 + ranges[axis].1) / 2.0;
                            if wide > 0.0 {
                                (p[axis] - middle) / wide * EXTENT
                            } else {
                                0.0
                            }
                        };
                        [fit(0), fit(1), fit(2)]
                    })
                    .collect()
            }
            Some("layers") => layered_layout(&group_of, EXTENT * 0.4),
            _ => sphere_layout(nodes.len(), EXTENT * 0.5),
        };
        let mut scene = DrawnScene::new(SceneKind::Network);
        let mut segments = Vec::with_capacity(links.len() * 2);
        for (from, to) in &links {
            segments.push(placed[*from]);
            segments.push(placed[*to]);
        }
        if !segments.is_empty() {
            let mut lines = SceneObject::new(
                "links",
                SceneShape::Line {
                    points: flat(&segments)?,
                    segments: true,
                    smooth: false,
                    arrow: adk_surface::genui::LineEnds::None,
                    dashed: false,
                },
            );
            lines.look.color = SceneColor::Neutral;
            scene.objects.push(lines);
        }
        scene.objects.push(SceneObject::new(
            "nodes",
            SceneShape::Points {
                at: flat(&placed)?,
                size: num(0.22)?,
                series: group_of
                    .iter()
                    .map(|group| u8::try_from(group % MAX_SERIES).unwrap_or(7))
                    .collect(),
            },
        ));
        let mut busiest: Vec<usize> = (0..nodes.len()).collect();
        busiest.sort_by(|a, b| degree[*b].cmp(&degree[*a]).then(a.cmp(b)));
        for at in busiest.iter().take(MAX_LABELS) {
            let node = &nodes[*at];
            let text = node
                .get("label")
                .and_then(Value::as_str)
                .map_or_else(|| cell(node.get("id")), str::to_string);
            let p = placed[*at];
            scene.objects.push(label(
                format!("node-{at}"),
                &text,
                [p[0], p[1] + 0.45, p[2]],
                LabelSize::Small,
            )?);
        }
        scene.camera.spin = graph.get("spin") == Some(&Value::Bool(true));
        let name = |at: usize| {
            let node = &nodes[at];
            short(
                &node
                    .get("label")
                    .and_then(Value::as_str)
                    .map_or_else(|| cell(node.get("id")), str::to_string),
            )
        };
        let mut words = vec![format!("{} nodes and {} links", nodes.len(), links.len())];
        words.extend(busiest.iter().map(|at| {
            let count = degree[*at];
            format!(
                "{}: {count} {}",
                name(*at),
                if count == 1 { "link" } else { "links" }
            )
        }));
        Ok(finish(graph, scene, Vec::new(), Some(text_view(words))))
    }
}

/// Places on a globe and the routes between them.
pub(crate) struct GlobeComponent;

impl ExtensionComponent for GlobeComponent {
    fn name(&self) -> &'static str {
        "Globe"
    }

    fn schema(&self) -> Value {
        let place = json!({
            "type": "object",
            "properties": {
                "lat": { "type": "number", "minimum": -90, "maximum": 90 },
                "lon": { "type": "number", "minimum": -180, "maximum": 180 },
                "label": text(),
                "group": key()
            },
            "required": ["lat", "lon"],
            "additionalProperties": false
        });
        let end =
            json!({ "type": "array", "items": { "type": "number" }, "minItems": 2, "maxItems": 2 });
        let route = json!({
            "type": "object",
            "properties": { "from": end, "to": end, "label": text() },
            "required": ["from", "to"],
            "additionalProperties": false
        });
        component(
            "Globe",
            view(json!({
                "points": data(place, MAX_GLOBE_POINTS),
                "arcs": data(route, MAX_GLOBE_ARCS),
                "focus": {
                    "type": "object",
                    "properties": {
                        "lat": { "type": "number", "minimum": -90, "maximum": 90 },
                        "lon": { "type": "number", "minimum": -180, "maximum": 180 }
                    },
                    "required": ["lat", "lon"],
                    "additionalProperties": false
                },
                "spin": { "type": "boolean" }
            })),
            &["points"],
        )
    }

    fn usage(&self) -> &'static str {
        "Globe: {\"component\": \"Globe\", \"title\", \"points\": [{\"lat\", \"lon\", \"label\", \
         \"group\"}] (at most 500), \"arcs\": [{\"from\": [lat, lon], \"to\": [lat, lon], \
         \"label\"}] (at most 200), \"focus\": {\"lat\", \"lon\"}, \"spin\"}: places and routes."
    }

    fn home(&self, globe: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let points = rows(globe, "points", MAX_GLOBE_POINTS)?;
        let arcs = match globe.get("arcs") {
            Some(_) => rows(globe, "arcs", MAX_GLOBE_ARCS)?,
            None => &[],
        };
        let place = |value: Option<&Value>, what: &str| -> Result<(f64, f64), String> {
            let (lat, lon) = match value {
                Some(Value::Array(pair)) if pair.len() == 2 => {
                    (finite(pair.first(), what)?, finite(pair.get(1), what)?)
                }
                Some(object @ Value::Object(_)) => (
                    finite(object.get("lat"), what)?,
                    finite(object.get("lon"), what)?,
                ),
                _ => return Err(format!("{what} is not a place")),
            };
            if lat.abs() > 90.0 || lon.abs() > 180.0 {
                return Err(format!(
                    "{what} is at {lat}, {lon}: latitude ±90, longitude ±180"
                ));
            }
            Ok((lat, lon))
        };
        let mut scene = DrawnScene::new(SceneKind::Globe);
        let mut sphere = SceneObject::new(
            "globe",
            SceneShape::Globe {
                radius: num(GLOBE)?,
            },
        );
        // A plain globe, so the places and routes on it carry the colour.
        sphere.look.color = SceneColor::Neutral;
        scene.objects.push(sphere);
        let mut groups: BTreeMap<String, usize> = BTreeMap::new();
        let mut at = Vec::with_capacity(points.len());
        let mut series = Vec::with_capacity(points.len());
        let mut names = Vec::new();
        for (index, point) in points.iter().enumerate() {
            let (lat, lon) = place(Some(point), &format!("point {index}"))?;
            at.push(on_globe(lat, lon, GLOBE * 1.01));
            let next = groups.len();
            series.push(
                u8::try_from(*groups.entry(cell(point.get("group"))).or_insert(next) % MAX_SERIES)
                    .unwrap_or(7),
            );
            if let Some(text) = point.get("label").and_then(Value::as_str) {
                names.push(text.to_string());
                if names.len() <= MAX_LABELS {
                    let p = on_globe(lat, lon, GLOBE * 1.12);
                    scene
                        .objects
                        .push(label(format!("place-{index}"), text, p, LabelSize::Small)?);
                }
            }
        }
        if !at.is_empty() {
            scene.objects.push(SceneObject::new(
                "places",
                SceneShape::Points {
                    at: flat(&at)?,
                    size: num(0.13)?,
                    series,
                },
            ));
        }
        let mut routes = Vec::new();
        for (index, route) in arcs.iter().enumerate() {
            let from = place(route.get("from"), &format!("arc {index}'s start"))?;
            let to = place(route.get("to"), &format!("arc {index}'s end"))?;
            let mut path = SceneObject::new(
                format!("route-{index}"),
                line(&arc(from, to, GLOBE, ARC_SEGMENTS), false)?,
            );
            path.look.color = SceneColor::Chart2;
            if let Some(text) = route.get("label").and_then(Value::as_str) {
                path.label = Some(short(text));
                routes.push(text.to_string());
            }
            scene.objects.push(path);
        }
        if let Some(focus) = globe.get("focus") {
            let (lat, lon) = place(Some(focus), "focus")?;
            scene.camera = SceneCamera {
                at: Some(xyz(on_globe(lat, lon, GLOBE * 3.0))?),
                target: Some(xyz([0.0, 0.0, 0.0])?),
                ..SceneCamera::default()
            };
        }
        scene.camera.spin = globe.get("spin") == Some(&Value::Bool(true));
        scene.lighting = SceneLighting::Soft;
        let mut words = vec![format!("{} places and {} routes", points.len(), arcs.len())];
        words.extend(names.iter().map(|name| short(name)));
        words.extend(
            routes
                .iter()
                .map(|route| format!("Route: {}", short(route))),
        );
        Ok(finish(globe, scene, Vec::new(), Some(text_view(words))))
    }
}

/// One object of a spatial scene, as its schema allows.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ObjectSpec {
    id: String,
    #[serde(default)]
    parent: Option<String>,
    shape: String,
    #[serde(default)]
    position: Option<Value>,
    #[serde(default)]
    rotation: Option<Value>,
    #[serde(default)]
    scale: Option<Value>,
    #[serde(default)]
    points: Option<Vec<Value>>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    opacity: Option<f64>,
    #[serde(default)]
    wireframe: Option<bool>,
    #[serde(default)]
    finish: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    motion: Option<MotionSpec>,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct MotionSpec {
    kind: String,
    #[serde(default)]
    seconds: Option<f64>,
    #[serde(default)]
    axis: Option<String>,
    #[serde(default)]
    amount: Option<f64>,
    #[serde(default)]
    path: Option<Vec<Value>>,
}

fn object_schema() -> Value {
    let points =
        json!({ "type": "array", "items": vector(-1000.0, 1000.0), "minItems": 2, "maxItems": 64 });
    json!({
        "type": "object",
        "properties": {
            "id": key(),
            "parent": key(),
            "shape": one_of(&["group", "box", "sphere", "cylinder", "cone", "plane", "torus", "line", "arrow", "text"]),
            "position": vector(-1000.0, 1000.0),
            "rotation": vector(-360.0, 360.0),
            "scale": vector(0.001, 1000.0),
            "points": points,
            "text": text(),
            "color": one_of(COLORS),
            "state": one_of(STATES),
            "opacity": { "type": "number", "minimum": 0.15, "maximum": 1 },
            "wireframe": { "type": "boolean" },
            "finish": one_of(&["matte", "glossy", "flat"]),
            "label": text(),
            "motion": {
                "type": "object",
                "properties": {
                    "kind": one_of(&["spin", "bob", "pulse", "orbit", "path"]),
                    "seconds": { "type": "number", "minimum": 0.2, "maximum": 120 },
                    "axis": one_of(&["x", "y", "z"]),
                    "amount": { "type": "number", "minimum": 0, "maximum": 100 },
                    "path": points
                },
                "required": ["kind"],
                "additionalProperties": false
            },
            "value": { "type": "string", "minLength": 1, "maxLength": 200 }
        },
        "required": ["id", "shape"],
        "additionalProperties": false
    })
}

/// A scene laid out in space: shapes, groups, lines, arrows and text, lit and seen as the agent
/// says, with motion, panels beside objects and objects the person can pick.
pub(crate) struct SceneComponent;

impl ExtensionComponent for SceneComponent {
    fn name(&self) -> &'static str {
        "Scene"
    }

    fn schema(&self) -> Value {
        component(
            "Scene",
            view(json!({
                "objects": data(object_schema(), MAX_OBJECTS),
                "camera": {
                    "type": "object",
                    "properties": {
                        "preset": one_of(&["fit", "front", "top", "side", "iso"]),
                        "position": vector(-1000.0, 1000.0),
                        "target": vector(-1000.0, 1000.0),
                        "fov": { "type": "number", "minimum": 15, "maximum": 90 },
                        "spin": { "type": "boolean" }
                    },
                    "additionalProperties": false
                },
                "lighting": one_of(&["studio", "soft", "flat"]),
                "lights": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "kind": one_of(&["point", "spot", "directional"]),
                            "position": vector(-1000.0, 1000.0),
                            "color": one_of(COLORS),
                            "intensity": { "type": "number", "minimum": 0, "maximum": 2 }
                        },
                        "required": ["kind", "position"],
                        "additionalProperties": false
                    },
                    "maxItems": MAX_LIGHTS
                },
                "pickInto": { "type": "string", "pattern": "^/[^\\s]*$", "maxLength": 200 },
                "panels": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": { "anchor": key(), "child": component_id() },
                        "required": ["anchor", "child"],
                        "additionalProperties": false
                    },
                    "maxItems": MAX_PANELS
                }
            })),
            &["objects"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[ChildSlot::EachOf {
            array: "panels",
            child: "child",
        }]
    }

    fn usage(&self) -> &'static str {
        "Scene: {\"component\": \"Scene\", \"title\", \"objects\": [{\"id\", \"parent\", \"shape\": \
         \"group|box|sphere|cylinder|cone|plane|torus|line|arrow|text\", \"position\", \"rotation\" \
         (degrees), \"scale\", \"points\" (line, arrow), \"text\" (text), \"color\": \"chart1..chart8|\
         ink|neutral|muted\", \"state\": \"ok|warning|critical|info\", \"label\", \"motion\": {\"kind\": \
         \"spin|bob|pulse|orbit|path\", \"seconds\"}, \"value\"}] (at most 128), \"camera\", \
         \"lighting\", \"pickInto\": \"/path of a ChoicePicker you show\", \"panels\": [{\"anchor\": \
         \"<object id>\", \"child\": \"<id of a Text, Metric or KeyValue>\"}]}."
    }

    fn home(&self, spec: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let written = rows(spec, "objects", MAX_OBJECTS)?;
        let mut scene = DrawnScene::new(SceneKind::Space);
        let mut moving = 0;
        for (index, written) in written.iter().enumerate() {
            let object: ObjectSpec = serde_json::from_value(written.clone())
                .map_err(|error| format!("object {index}: {error}"))?;
            let id = object.id.clone();
            let shape = match object.shape.as_str() {
                "line" | "arrow" => {
                    let points = object
                        .points
                        .as_deref()
                        .ok_or_else(|| format!("{id} is a line with no points"))?
                        .iter()
                        .map(|p| point(Some(p), &format!("a point of {id}")))
                        .collect::<Result<Vec<_>, _>>()?;
                    if !(2..=64).contains(&points.len()) {
                        return Err(format!("{id} has {} points; 2 to 64", points.len()));
                    }
                    line(&points, object.shape == "arrow")?
                }
                "text" => SceneShape::Label {
                    text: short(
                        object
                            .text
                            .as_deref()
                            .filter(|text| !text.trim().is_empty())
                            .ok_or_else(|| format!("{id} is text with no text"))?,
                    ),
                    size: LabelSize::Medium,
                },
                other => unit_shape(other)
                    .ok_or_else(|| format!("{id} has an unknown shape '{other}'"))?,
            };
            let mut drawn = SceneObject::new(id.clone(), shape);
            drawn.parent = object.parent;
            if let Some(position) = &object.position {
                drawn.at = xyz(point(Some(position), &format!("{id}'s position"))?)?;
            }
            if let Some(rotation) = &object.rotation {
                drawn.turn = Some(xyz(point(Some(rotation), &format!("{id}'s rotation"))?)?);
            }
            if let Some(size) = &object.scale {
                drawn.scale = Some(xyz(point(Some(size), &format!("{id}'s scale"))?)?);
            }
            drawn.look = SceneLook {
                color: object
                    .color
                    .as_deref()
                    .map_or(Ok(SceneColor::Chart1), |name| {
                        color(name).ok_or_else(|| {
                            format!("{id}'s colour '{name}' is not one of the palette's")
                        })
                    })?,
                opacity: object.opacity.map(num).transpose()?,
                wireframe: object.wireframe.unwrap_or(false),
                finish: match object.finish.as_deref() {
                    Some("glossy") => SceneFinish::Glossy,
                    Some("flat") => SceneFinish::Flat,
                    _ => SceneFinish::Matte,
                },
            };
            drawn.state = object
                .state
                .as_deref()
                .map(|name| {
                    state(name).ok_or_else(|| {
                        format!("{id}'s state '{name}' is not ok, warning, critical or info")
                    })
                })
                .transpose()?;
            drawn.label = object.label.map(|label| short(&label));
            drawn.value = object.value;
            if let Some(motion) = object.motion {
                moving += 1;
                if moving > MAX_MOVING {
                    return Err(format!("more than {MAX_MOVING} objects move"));
                }
                scene.motion.push(SceneMotion {
                    target: id.clone(),
                    kind: match motion.kind.as_str() {
                        "spin" => MotionKind::Spin,
                        "bob" => MotionKind::Bob,
                        "pulse" => MotionKind::Pulse,
                        "orbit" => MotionKind::Orbit,
                        "path" => MotionKind::Path,
                        other => return Err(format!("{id} moves as '{other}'")),
                    },
                    axis: match motion.axis.as_deref() {
                        Some("x") => Axis::X,
                        Some("z") => Axis::Z,
                        _ => Axis::Y,
                    },
                    seconds: num(motion.seconds.unwrap_or(6.0))?,
                    amount: motion.amount.map(num).transpose()?,
                    delay: None,
                    repeat: None,
                    path: match motion.path {
                        Some(path) => flat(
                            &path
                                .iter()
                                .map(|p| point(Some(p), &format!("{id}'s path")))
                                .collect::<Result<Vec<_>, _>>()?,
                        )?,
                        None => Vec::new(),
                    },
                });
            }
            scene.objects.push(drawn);
        }
        if let Some(camera) = spec.get("camera") {
            scene.camera = SceneCamera {
                preset: preset(camera.get("preset").and_then(Value::as_str)),
                at: camera
                    .get("position")
                    .map(|p| point(Some(p), "the camera's position").and_then(xyz))
                    .transpose()?,
                target: camera
                    .get("target")
                    .map(|p| point(Some(p), "the camera's target").and_then(xyz))
                    .transpose()?,
                fov: camera
                    .get("fov")
                    .and_then(Value::as_f64)
                    .map(num)
                    .transpose()?,
                fixed: false,
                spin: camera.get("spin") == Some(&Value::Bool(true)),
            };
        }
        scene.lighting = match spec.str("lighting") {
            Some("soft") => SceneLighting::Soft,
            Some("flat") => SceneLighting::Flat,
            _ => SceneLighting::Studio,
        };
        for light in spec
            .get("lights")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .take(MAX_LIGHTS)
        {
            scene.lights.push(SceneLight {
                kind: match light.get("kind").and_then(Value::as_str) {
                    Some("spot") => LightKind::Spot,
                    Some("directional") => LightKind::Directional,
                    _ => LightKind::Point,
                },
                at: xyz(point(light.get("position"), "a light's position")?)?,
                color: light.get("color").and_then(Value::as_str).and_then(color),
                intensity: num(light
                    .get("intensity")
                    .and_then(Value::as_f64)
                    .unwrap_or(1.0))?,
            });
        }
        if let Some(path) = spec.str("pickInto") {
            scene.select = Some(SceneSelect {
                path: path.to_string(),
                note: String::new(),
            });
        }
        let panels = spec
            .get("panels")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
            .map(|(index, panel)| HomePanel {
                anchor: cell(panel.get("anchor")),
                content: HomeNode::Child(index),
            })
            .collect::<Vec<_>>();
        if panels.len() > MAX_PANELS {
            return Err(format!(
                "it has {} panels; at most {MAX_PANELS}",
                panels.len()
            ));
        }
        Ok(finish(spec, scene, panels, None))
    }
}

/// A card with depth: it tilts towards the pointer, turns over to its back, or stacks layers.
pub(crate) struct DepthCardComponent;

impl ExtensionComponent for DepthCardComponent {
    fn name(&self) -> &'static str {
        "DepthCard"
    }

    fn schema(&self) -> Value {
        component(
            "DepthCard",
            json!({
                "title": text(),
                "effect": one_of(&["tilt", "flip", "layers"]),
                "front": component_id(),
                "back": component_id(),
                "layers": { "type": "array", "items": component_id(), "minItems": 1, "maxItems": MAX_LAYERS }
            }),
            &["front"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[
            ChildSlot::One("front"),
            ChildSlot::One("back"),
            ChildSlot::Many("layers"),
        ]
    }

    fn usage(&self) -> &'static str {
        "DepthCard: {\"component\": \"DepthCard\", \"title\", \"effect\": \"tilt|flip|layers\", \
         \"front\": \"<id>\", \"back\": \"<id>\" (flip), \"layers\": [ids] (layers, at most 4)}: a \
         card with depth; the home turns a flip card with its own button."
    }

    fn home(&self, card: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let has_back = card.str("back").is_some();
        let layers = card
            .get("layers")
            .and_then(Value::as_array)
            .map_or(0, Vec::len);
        let effect = match card.str("effect").unwrap_or("tilt") {
            "flip" if has_back && layers == 0 => DepthEffect::Flip,
            "flip" => return Err("a flip card has a front and a back, and no layers".into()),
            "layers" if layers > 0 && !has_back => DepthEffect::Layers,
            "layers" => return Err("a layered card has a front and layers, and no back".into()),
            _ if has_back || layers > 0 => {
                return Err("a tilting card has only a front".into());
            }
            _ => DepthEffect::Tilt,
        };
        let depth = HomeNode::Depth {
            depth: DrawnDepth {
                effect,
                titles: Vec::new(),
            },
            children: (0..card.children).map(HomeNode::Child).collect(),
        };
        Ok(titled(card.str("title"), None, depth))
    }
}

/// Components on a stage the person turns, a curved wall or a carousel.
pub(crate) struct StageComponent;

impl ExtensionComponent for StageComponent {
    fn name(&self) -> &'static str {
        "Stage"
    }

    fn schema(&self) -> Value {
        component(
            "Stage",
            json!({
                "title": text(),
                "description": text(),
                "layout": one_of(&["stage", "wall", "carousel"]),
                "children": { "type": "array", "items": component_id(), "minItems": 1, "maxItems": MAX_STAGED }
            }),
            &["children"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[ChildSlot::Many("children")]
    }

    fn usage(&self) -> &'static str {
        "Stage: {\"component\": \"Stage\", \"title\", \"description\", \"layout\": \
         \"stage|wall|carousel\", \"children\": [ids] (1 to 12)}: components arranged in 3D; a \
         stage or wall holds only what shows (charts, metrics, text), a carousel may hold a form."
    }

    fn home(&self, stage: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let effect = match stage.str("layout") {
            Some("wall") => DepthEffect::Wall,
            Some("carousel") => DepthEffect::Carousel,
            _ => DepthEffect::Stage,
        };
        if !(1..=MAX_STAGED).contains(&stage.children) {
            return Err(format!(
                "it presents {} components; 1 to {MAX_STAGED}",
                stage.children
            ));
        }
        let depth = HomeNode::Depth {
            depth: DrawnDepth {
                effect,
                titles: Vec::new(),
            },
            children: (0..stage.children).map(HomeNode::Child).collect(),
        };
        Ok(titled(stage.str("title"), stage.str("description"), depth))
    }
}
