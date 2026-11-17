//! What adk-ui's 3D components project to: `genui`'s home-native [`DrawnScene`], built from
//! checked properties. Geometry is worked out here, deterministically, so the home only draws:
//! layouts, globe positions and arcs, scales and ticks.

use adk_surface::genui::{
    Axis, DrawnScene, HomeNode, HomeText, LabelSize, LineEnds, SceneColor, SceneObject, SceneShape,
    SceneState, SceneTick, Xyz,
};
use serde_json::{Number, Value};

use crate::insight::format::number as compact;

/// The most labels a data-viz scene shows; the rest are in its text view.
pub(crate) const MAX_LABELS: usize = 24;
/// The most lines a scene's text view lists; the rest are counted.
pub(crate) const TEXT_LINES: usize = 24;
/// The longest label a scene shows, in characters.
const LABEL_CHARS: usize = 60;
/// The largest a data-viz scene draws along any axis.
pub(crate) const EXTENT: f64 = 8.0;

/// `value` as a scene number.
pub(crate) fn num(value: f64) -> Result<Number, String> {
    Number::from_f64(value).ok_or_else(|| format!("{value} is not a finite number"))
}

/// `[x, y, z]` as scene numbers.
pub(crate) fn xyz(point: [f64; 3]) -> Result<Xyz, String> {
    Ok([num(point[0])?, num(point[1])?, num(point[2])?])
}

/// Points as a flat list of scene numbers.
pub(crate) fn flat(points: &[[f64; 3]]) -> Result<Vec<Number>, String> {
    points
        .iter()
        .flat_map(|point| point.iter())
        .map(|value| num(*value))
        .collect()
}

/// A number from a property or a row: a JSON number, finite.
pub(crate) fn finite(value: Option<&Value>, what: &str) -> Result<f64, String> {
    let value = value.ok_or_else(|| format!("{what} is missing"))?;
    value
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| format!("{what} is not a number"))
}

/// A point `[x, y, z]` from a property.
pub(crate) fn point(value: Option<&Value>, what: &str) -> Result<[f64; 3], String> {
    let items = value
        .and_then(Value::as_array)
        .filter(|items| items.len() == 3)
        .ok_or_else(|| format!("{what} is not three numbers"))?;
    Ok([
        finite(items.first(), what)?,
        finite(items.get(1), what)?,
        finite(items.get(2), what)?,
    ])
}

/// A label shortened to what a scene shows, on a word boundary when it can be.
pub(crate) fn short(label: &str) -> String {
    let label = label.trim();
    if label.chars().count() <= LABEL_CHARS {
        return label.to_string();
    }
    let cut: String = label.chars().take(LABEL_CHARS - 1).collect();
    format!("{}…", cut.trim_end())
}

/// A string from a row: its text, or a number written out.
pub(crate) fn cell(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        _ => String::new(),
    }
}

/// A palette colour by its name in a schema.
pub(crate) fn color(name: &str) -> Option<SceneColor> {
    Some(match name {
        "chart1" => SceneColor::Chart1,
        "chart2" => SceneColor::Chart2,
        "chart3" => SceneColor::Chart3,
        "chart4" => SceneColor::Chart4,
        "chart5" => SceneColor::Chart5,
        "chart6" => SceneColor::Chart6,
        "chart7" => SceneColor::Chart7,
        "chart8" => SceneColor::Chart8,
        "ink" => SceneColor::Ink,
        "neutral" => SceneColor::Neutral,
        "muted" => SceneColor::Muted,
        _ => return None,
    })
}

/// The palette colours a schema offers.
pub(crate) const COLORS: &[&str] = &[
    "chart1", "chart2", "chart3", "chart4", "chart5", "chart6", "chart7", "chart8", "ink",
    "neutral", "muted",
];

/// A state by its name in a schema.
pub(crate) fn state(name: &str) -> Option<SceneState> {
    Some(match name {
        "ok" => SceneState::Ok,
        "warning" => SceneState::Warning,
        "critical" => SceneState::Critical,
        "info" => SceneState::Info,
        _ => return None,
    })
}

/// The states a schema offers.
pub(crate) const STATES: &[&str] = &["ok", "warning", "critical", "info"];

/// The chart colour nearest `hex` (`#rrggbb`) by hue, for a studio scene's own colours: the
/// home draws in its palette, never in a colour an agent picks.
pub(crate) fn nearest_chart(hex: &str) -> SceneColor {
    let channel =
        |at: usize| u8::from_str_radix(hex.get(at..at + 2).unwrap_or("00"), 16).unwrap_or(0);
    let (r, g, b) = (channel(1), channel(3), channel(5));
    let (r, g, b) = (
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    );
    let high = r.max(g).max(b);
    let low = r.min(g).min(b);
    if high - low < 0.08 {
        return if high > 0.5 {
            SceneColor::Neutral
        } else {
            SceneColor::Ink
        };
    }
    let hue = if high == r {
        60.0 * (((g - b) / (high - low)).rem_euclid(6.0))
    } else if high == g {
        60.0 * ((b - r) / (high - low) + 2.0)
    } else {
        60.0 * ((r - g) / (high - low) + 4.0)
    };
    // The light palette's hues: blue, teal, violet, pink, green, cyan, slate, magenta.
    const HUES: [(f64, SceneColor); 7] = [
        (221.0, SceneColor::Chart1),
        (175.0, SceneColor::Chart2),
        (262.0, SceneColor::Chart3),
        (330.0, SceneColor::Chart4),
        (85.0, SceneColor::Chart5),
        (192.0, SceneColor::Chart6),
        (295.0, SceneColor::Chart8),
    ];
    let distance = |to: f64| {
        let d = (hue - to).abs();
        d.min(360.0 - d)
    };
    // Reds, oranges and yellows have no chart colour; they read as the nearest warm one.
    if !(40.0..=345.0).contains(&hue) {
        return SceneColor::Chart4;
    }
    HUES.iter()
        .min_by(|a, b| distance(a.0).total_cmp(&distance(b.0)))
        .map_or(SceneColor::Chart1, |(_, color)| *color)
}

/// A unit shape of a primitive's name, sized by its object's scale.
pub(crate) fn unit_shape(name: &str) -> Option<SceneShape> {
    let n = |value: f64| Number::from_f64(value).expect("a finite constant");
    Some(match name {
        "box" => SceneShape::Box {
            size: [n(1.0), n(1.0), n(1.0)],
        },
        "sphere" => SceneShape::Sphere { radius: n(0.5) },
        "cylinder" => SceneShape::Cylinder {
            radius_top: n(0.5),
            radius_bottom: n(0.5),
            height: n(1.0),
        },
        "cone" => SceneShape::Cylinder {
            radius_top: n(0.0),
            radius_bottom: n(0.5),
            height: n(1.0),
        },
        "plane" => SceneShape::Plane {
            width: n(1.0),
            depth: n(1.0),
        },
        "torus" => SceneShape::Torus {
            radius: n(0.4),
            tube: n(0.15),
        },
        "group" => SceneShape::Group,
        _ => return None,
    })
}

/// A line through `points`, with an arrowhead at its end when `arrow`.
pub(crate) fn line(points: &[[f64; 3]], arrow: bool) -> Result<SceneShape, String> {
    Ok(SceneShape::Line {
        points: flat(points)?,
        segments: false,
        smooth: false,
        arrow: if arrow { LineEnds::End } else { LineEnds::None },
        dashed: false,
    })
}

/// Text facing the viewer at `at`.
pub(crate) fn label(
    id: String,
    text: &str,
    at: [f64; 3],
    size: LabelSize,
) -> Result<SceneObject, String> {
    Ok(SceneObject::new(
        id,
        SceneShape::Label {
            text: short(text),
            size,
        },
    )
    .at(xyz(at)?))
}

/// Where `(lat, lon)` in degrees is on a globe of `radius`; y is north.
pub(crate) fn on_globe(lat: f64, lon: f64, radius: f64) -> [f64; 3] {
    let (lat, lon) = (lat.to_radians(), lon.to_radians());
    [
        radius * lat.cos() * lon.sin(),
        radius * lat.sin(),
        radius * lat.cos() * lon.cos(),
    ]
}

/// The great-circle arc between two places on a globe of `radius`, lifted towards its middle
/// so it reads above the surface: `segments` + 1 points.
pub(crate) fn arc(from: (f64, f64), to: (f64, f64), radius: f64, segments: usize) -> Vec<[f64; 3]> {
    let a = on_globe(from.0, from.1, 1.0);
    let b = on_globe(to.0, to.1, 1.0);
    let dot = (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let lift = 0.08 + 0.25 * angle / std::f64::consts::PI;
    (0..=segments)
        .map(|step| {
            let t = step as f64 / segments as f64;
            let (wa, wb) = if angle < 1e-6 {
                (1.0 - t, t)
            } else {
                (
                    ((1.0 - t) * angle).sin() / angle.sin(),
                    (t * angle).sin() / angle.sin(),
                )
            };
            let point = [
                wa * a[0] + wb * b[0],
                wa * a[1] + wb * b[1],
                wa * a[2] + wb * b[2],
            ];
            let length = (point[0].powi(2) + point[1].powi(2) + point[2].powi(2))
                .sqrt()
                .max(1e-9);
            let height = radius * (1.0 + lift * (std::f64::consts::PI * t).sin());
            [
                point[0] / length * height,
                point[1] / length * height,
                point[2] / length * height,
            ]
        })
        .collect()
}

/// `count` points spread evenly over a sphere of `radius` (a Fibonacci lattice).
pub(crate) fn sphere_layout(count: usize, radius: f64) -> Vec<[f64; 3]> {
    let golden = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (0..count)
        .map(|index| {
            if count == 1 {
                return [0.0, 0.0, 0.0];
            }
            let y = 1.0 - 2.0 * index as f64 / (count - 1) as f64;
            let ring = (1.0 - y * y).max(0.0).sqrt();
            let theta = golden * index as f64;
            [
                radius * ring * theta.cos(),
                radius * y,
                radius * ring * theta.sin(),
            ]
        })
        .collect()
}

/// Each layer on its own ring, one above another: `layers[i]` is the layer of node `i`.
pub(crate) fn layered_layout(layers: &[usize], radius: f64) -> Vec<[f64; 3]> {
    let count = layers.iter().max().map_or(0, |top| top + 1);
    let mut sizes = vec![0usize; count];
    for layer in layers {
        sizes[*layer] += 1;
    }
    let mut placed = vec![0usize; count];
    layers
        .iter()
        .map(|layer| {
            let index = placed[*layer];
            placed[*layer] += 1;
            let around = std::f64::consts::TAU * index as f64 / sizes[*layer].max(1) as f64;
            let y = if count <= 1 {
                0.0
            } else {
                EXTENT * 0.6 * (0.5 - *layer as f64 / (count - 1) as f64)
            };
            let ring = if sizes[*layer] == 1 { 0.0 } else { radius };
            [ring * around.cos(), y, ring * around.sin()]
        })
        .collect()
}

/// Where `value` falls on an axis of `length`, centred, between `low` and `high`.
pub(crate) fn scale(value: f64, low: f64, high: f64, length: f64) -> f64 {
    if high > low {
        (value - low) / (high - low) * length - length / 2.0
    } else {
        0.0
    }
}

/// Three ticks along `axis`: the low end, the middle and the high end, worded compactly.
pub(crate) fn ticks(
    axis: Axis,
    low: f64,
    high: f64,
    length: f64,
) -> Result<Vec<SceneTick>, String> {
    let middle = (low + high) / 2.0;
    [low, middle, high]
        .into_iter()
        .map(|value| {
            Ok(SceneTick {
                axis,
                at: num(scale(value, low, high, length))?,
                text: compact(value),
            })
        })
        .collect()
}

/// The lowest and highest of `values`.
pub(crate) fn range(values: impl IntoIterator<Item = f64>) -> Option<(f64, f64)> {
    values.into_iter().fold(None, |range, value| match range {
        None => Some((value, value)),
        Some((low, high)) => Some((low.min(value), high.max(value))),
    })
}

/// A text view: a line per entry, at most [`TEXT_LINES`], then how many more.
pub(crate) fn text_view(lines: Vec<String>) -> HomeNode {
    let total = lines.len();
    let mut nodes: Vec<HomeNode> = lines
        .into_iter()
        .take(TEXT_LINES)
        .map(HomeNode::text)
        .collect();
    if total > TEXT_LINES {
        nodes.push(HomeNode::Text {
            text: format!("and {} more", total - TEXT_LINES),
            variant: HomeText::Caption,
        });
    }
    HomeNode::Column(nodes)
}

/// A heading and a caption, when given, above `body`.
pub(crate) fn titled(title: Option<&str>, description: Option<&str>, body: HomeNode) -> HomeNode {
    let mut nodes: Vec<HomeNode> = Vec::new();
    if let Some(title) = title.map(str::trim).filter(|title| !title.is_empty()) {
        nodes.push(HomeNode::heading(4, title));
    }
    if let Some(description) = description.map(str::trim).filter(|text| !text.is_empty()) {
        nodes.push(HomeNode::caption(description));
    }
    if nodes.is_empty() {
        return body;
    }
    nodes.push(body);
    HomeNode::Column(nodes)
}

/// A scene's height and controls from its component's properties.
pub(crate) fn framed(scene: &mut DrawnScene, height: Option<u64>, controls: Option<&str>) {
    scene.height = height.and_then(|height| u16::try_from(height).ok());
    if controls == Some("none") {
        scene.camera.fixed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn studio_colours_read_as_the_nearest_chart_colour() {
        assert_eq!(nearest_chart("#2563eb"), SceneColor::Chart1);
        assert_eq!(nearest_chart("#0d9488"), SceneColor::Chart2);
        assert_eq!(nearest_chart("#7c3aed"), SceneColor::Chart3);
        assert_eq!(
            nearest_chart("#c03a00"),
            SceneColor::Chart4,
            "never the decision orange"
        );
        assert_eq!(nearest_chart("#888888"), SceneColor::Neutral);
        assert_eq!(nearest_chart("#111111"), SceneColor::Ink);
    }

    #[test]
    fn places_and_routes_sit_on_and_over_the_globe() {
        let north = on_globe(90.0, 0.0, 4.0);
        assert!((north[1] - 4.0).abs() < 1e-9);
        let route = arc((-1.29, 36.82), (51.5, -0.12), 4.0, 16);
        assert_eq!(route.len(), 17);
        let length = |p: [f64; 3]| (p[0].powi(2) + p[1].powi(2) + p[2].powi(2)).sqrt();
        assert!((length(route[0]) - 4.0).abs() < 1e-9);
        assert!(length(route[8]) > 4.0, "the middle is lifted");
    }

    #[test]
    fn layouts_are_deterministic_and_spread() {
        let points = sphere_layout(10, 5.0);
        assert_eq!(points, sphere_layout(10, 5.0));
        assert!(
            points
                .iter()
                .all(|p| ((p[0].powi(2) + p[1].powi(2) + p[2].powi(2)).sqrt() - 5.0).abs() < 1e-9)
        );
        let layered = layered_layout(&[0, 1, 1, 2], 3.0);
        assert!(layered[0][1] > layered[1][1] && layered[1][1] > layered[3][1]);
    }
}
