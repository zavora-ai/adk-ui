//! Builders for the spatial components: the same JSON an agent writes, in Rust.

use serde_json::{Value, json};

use crate::build::{Props, component_builder};

/// Bars over two categories.
///
/// ```rust
/// use adk_ui::spatial::Bar3D;
///
/// let bars = Bar3D::new("sales", serde_json::json!({ "path": "/sales" }), "quarter")
///     .series("north", "North")
///     .series("south", "South")
///     .title("Sales by region")
///     .build();
/// assert_eq!(bars["series"][1]["label"], "South");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Bar3D(Props);

impl Bar3D {
    /// Bars of `data` (rows, or a `{"path"}` binding), one row per `x_key` category.
    pub fn new(id: impl Into<String>, data: impl Into<Value>, x_key: impl Into<String>) -> Self {
        let mut props = Props::new(id, "Bar3D");
        props.set("data", data);
        props.set("x", json!({ "key": x_key.into() }));
        Self(props)
    }

    /// A series: the row key it reads and its label.
    #[must_use]
    pub fn series(mut self, key: impl Into<String>, label: impl Into<Value>) -> Self {
        self.0.push(
            "series",
            json!({ "key": key.into(), "label": label.into() }),
        );
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// The name of the value the bars stand for.
    #[must_use]
    pub fn y_label(mut self, label: impl Into<Value>) -> Self {
        self.0.set("yLabel", label);
        self
    }
}

component_builder!(Bar3D);

/// Points over three measures.
#[derive(Debug, Clone, PartialEq)]
pub struct Scatter3D(Props);

impl Scatter3D {
    /// Points of `data`, placed by the keys `x`, `y` and `z`.
    pub fn new(id: impl Into<String>, data: impl Into<Value>, keys: [&str; 3]) -> Self {
        let mut props = Props::new(id, "Scatter3D");
        props.set("data", data);
        for (axis, key) in ["x", "y", "z"].into_iter().zip(keys) {
            props.set(axis, json!({ "key": key }));
        }
        Self(props)
    }

    /// Coloured by the row key `key`.
    #[must_use]
    pub fn group(mut self, key: impl Into<String>) -> Self {
        self.0.set("group", key.into());
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(Scatter3D);

/// A value over a grid.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePlot(Props);

impl SurfacePlot {
    /// The grid `values`: rows of numbers, or a `{"path"}` binding.
    pub fn new(id: impl Into<String>, values: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "SurfacePlot");
        props.set("values", values);
        Self(props)
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(SurfacePlot);

/// Nodes and the links between them.
#[derive(Debug, Clone, PartialEq)]
pub struct Graph3D(Props);

impl Graph3D {
    /// A network of `nodes` and `edges` (lists, or `{"path"}` bindings).
    pub fn new(id: impl Into<String>, nodes: impl Into<Value>, edges: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Graph3D");
        props.set("nodes", nodes);
        props.set("edges", edges);
        Self(props)
    }

    /// Its groups stacked in layers.
    #[must_use]
    pub fn layers(mut self) -> Self {
        self.0.set("layout", "layers");
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(Graph3D);

/// Places on a globe and the routes between them.
#[derive(Debug, Clone, PartialEq)]
pub struct Globe(Props);

impl Globe {
    /// A globe of `points` (a list, or a `{"path"}` binding).
    pub fn new(id: impl Into<String>, points: impl Into<Value>) -> Self {
        let mut props = Props::new(id, "Globe");
        props.set("points", points);
        Self(props)
    }

    /// The routes between places.
    #[must_use]
    pub fn arcs(mut self, arcs: impl Into<Value>) -> Self {
        self.0.set("arcs", arcs);
        self
    }

    /// Seen from above `lat`, `lon`.
    #[must_use]
    pub fn focus(mut self, lat: f64, lon: f64) -> Self {
        self.0.set("focus", json!({ "lat": lat, "lon": lon }));
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(Globe);

/// How a scene object moves.
#[derive(Debug, Clone, PartialEq)]
pub struct Motion(Value);

impl Motion {
    /// Turns about its axis once every `seconds`.
    pub fn spin(seconds: f64) -> Self {
        Self(json!({ "kind": "spin", "seconds": seconds }))
    }

    /// Grows and shrinks once every `seconds`.
    pub fn pulse(seconds: f64) -> Self {
        Self(json!({ "kind": "pulse", "seconds": seconds }))
    }

    /// Swings `amount` up and down once every `seconds`.
    pub fn bob(seconds: f64, amount: f64) -> Self {
        Self(json!({ "kind": "bob", "seconds": seconds, "amount": amount }))
    }

    /// Circles its group's centre once every `seconds`.
    pub fn orbit(seconds: f64) -> Self {
        Self(json!({ "kind": "orbit", "seconds": seconds }))
    }
}

/// One object of a [`Scene`].
#[derive(Debug, Clone, PartialEq)]
pub struct Object(Value);

impl Object {
    /// A `shape` (`box`, `sphere`, `cylinder`, `cone`, `plane`, `torus` or `group`) at
    /// `position`, a unit in size until scaled.
    pub fn new(id: impl Into<String>, shape: &str, position: [f64; 3]) -> Self {
        Self(json!({ "id": id.into(), "shape": shape, "position": position }))
    }

    /// Text facing the viewer at `position`.
    pub fn text(id: impl Into<String>, text: impl Into<Value>, position: [f64; 3]) -> Self {
        Self(json!({ "id": id.into(), "shape": "text", "text": text.into(), "position": position }))
    }

    /// A line, or an arrow ending at its last point.
    pub fn line(id: impl Into<String>, points: &[[f64; 3]], arrow: bool) -> Self {
        Self(
            json!({ "id": id.into(), "shape": if arrow { "arrow" } else { "line" }, "points": points }),
        )
    }

    /// Its label, shown beside it.
    #[must_use]
    pub fn label(mut self, label: impl Into<Value>) -> Self {
        self.0["label"] = label.into();
        self
    }

    /// In the group `parent`.
    #[must_use]
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.0["parent"] = Value::String(parent.into());
        self
    }

    /// Its size along each axis.
    #[must_use]
    pub fn scale(mut self, scale: [f64; 3]) -> Self {
        self.0["scale"] = json!(scale);
        self
    }

    /// A palette colour: `chart1` to `chart8`, `ink`, `neutral` or `muted`.
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.0["color"] = Value::String(color.into());
        self
    }

    /// A state, `ok`, `warning`, `critical` or `info`, drawn in its colour beside its label.
    #[must_use]
    pub fn state(mut self, state: &str) -> Self {
        self.0["state"] = Value::String(state.into());
        self
    }

    /// How it moves.
    #[must_use]
    pub fn motion(mut self, motion: Motion) -> Self {
        self.0["motion"] = motion.0;
        self
    }

    /// The option value picking it chooses; it needs a label.
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0["value"] = Value::String(value.into());
        self
    }
}

/// A scene laid out in space.
///
/// ```rust
/// use adk_ui::spatial::{Motion, Object, Scene};
///
/// let yard = Scene::new("yard")
///     .object(Object::new("dock-3", "box", [3.0, 0.5, 0.0]).label("Dock 3").state("ok")
///         .motion(Motion::pulse(2.0)).value("dock-3"))
///     .pick_into("/dock")
///     .panel("dock-3", "dock-3-note")
///     .build();
/// assert_eq!(yard["pickInto"], "/dock");
/// assert_eq!(yard["panels"][0]["child"], "dock-3-note");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Scene(Props);

impl Scene {
    /// An empty scene; add objects.
    pub fn new(id: impl Into<String>) -> Self {
        Self(Props::new(id, "Scene"))
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// An object. At most 128.
    #[must_use]
    pub fn object(mut self, object: Object) -> Self {
        self.0.push("objects", object.0);
        self
    }

    /// Picking an object fills the choice field bound to `path`, which the drawing shows.
    #[must_use]
    pub fn pick_into(mut self, path: impl Into<String>) -> Self {
        self.0.set("pickInto", path.into());
        self
    }

    /// Shows the component `child` beside the object `anchor`.
    #[must_use]
    pub fn panel(mut self, anchor: impl Into<String>, child: impl Into<String>) -> Self {
        self.0.push(
            "panels",
            json!({ "anchor": anchor.into(), "child": child.into() }),
        );
        self
    }

    /// Seen from a preset: `fit`, `front`, `top`, `side` or `iso`.
    #[must_use]
    pub fn camera(mut self, preset: &str) -> Self {
        self.0.set("camera", json!({ "preset": preset }));
        self
    }
}

component_builder!(Scene);

/// A card with depth.
#[derive(Debug, Clone, PartialEq)]
pub struct DepthCard(Props);

impl DepthCard {
    /// A card showing `front`, which tilts towards the pointer.
    pub fn new(id: impl Into<String>, front: impl Into<String>) -> Self {
        let mut props = Props::new(id, "DepthCard");
        props.set("front", front.into());
        Self(props)
    }

    /// Turns over to show `back`.
    #[must_use]
    pub fn flip(mut self, back: impl Into<String>) -> Self {
        self.0.set("effect", "flip");
        self.0.set("back", back.into());
        self
    }

    /// Stacks `layers` behind the front.
    #[must_use]
    pub fn layers<I, S>(mut self, layers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.0.set("effect", "layers");
        self.0.set(
            "layers",
            layers
                .into_iter()
                .map(|layer| Value::String(layer.into()))
                .collect::<Vec<_>>(),
        );
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(DepthCard);

/// Components arranged in 3D.
#[derive(Debug, Clone, PartialEq)]
pub struct Stage(Props);

impl Stage {
    /// A stage the person turns, presenting `children`.
    pub fn new<I, S>(id: impl Into<String>, children: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut props = Props::new(id, "Stage");
        props.set(
            "children",
            children
                .into_iter()
                .map(|child| Value::String(child.into()))
                .collect::<Vec<_>>(),
        );
        Self(props)
    }

    /// On a curved wall.
    #[must_use]
    pub fn wall(mut self) -> Self {
        self.0.set("layout", "wall");
        self
    }

    /// On a carousel, one in front at a time.
    #[must_use]
    pub fn carousel(mut self) -> Self {
        self.0.set("layout", "carousel");
        self
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }
}

component_builder!(Stage);
