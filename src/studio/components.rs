//! Pages, regions and primitive 3D scenes as `genui` extension components, with builders.
//! Every one only shows: an app's navigation is the host's, never an action, and a scene's
//! objects carry no action. The containers declare the components they hold, so `genui`
//! checks every one of them as part of the drawing.

use std::sync::Arc;

use adk_surface::genui::{
    ChildSlot, DrawnScene, ExtensionComponent, HomeNode, HomeScene, ResolvedComponent, SceneKind,
    SceneObject as DrawnObject, SceneShape,
};
use serde_json::{Value, json};

use crate::build::{Props, component_builder};
use crate::scene;
use crate::schema::{TONES, component, component_id, key, one_of, text, vector};

/// The most pages an app has.
pub const MAX_PAGES: usize = 24;
/// The most objects a scene has.
pub const MAX_SCENE_OBJECTS: usize = 64;
/// The most components a page or region holds.
const MAX_HELD: usize = 32;

/// Every studio component beyond the insight ones, for
/// [`Catalog::extend`](adk_surface::genui::Catalog::extend).
pub fn components() -> Vec<Arc<dyn ExtensionComponent>> {
    vec![
        Arc::new(AppComponent),
        Arc::new(PageComponent),
        Arc::new(RegionComponent),
        Arc::new(SceneComponent),
    ]
}

fn heading(level: u8, text: Option<&str>) -> Option<HomeNode> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(|text| HomeNode::heading(level, text))
}

fn caption(text: Option<&str>) -> Option<HomeNode> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(HomeNode::caption)
}

fn held(component: &ResolvedComponent<'_>) -> impl Iterator<Item = HomeNode> {
    (0..component.children).map(HomeNode::Child)
}

/// An application of pages; the host navigates between them.
pub(crate) struct AppComponent;

impl ExtensionComponent for AppComponent {
    fn name(&self) -> &'static str {
        "App"
    }

    fn schema(&self) -> Value {
        component(
            "App",
            json!({
                "title": text(),
                "pages": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": { "id": key(), "title": text(), "child": component_id() },
                        "required": ["id", "title", "child"],
                        "additionalProperties": false
                    },
                    "minItems": 1,
                    "maxItems": MAX_PAGES
                },
                "navigation": one_of(&["top", "side"]),
                "initialPage": key()
            }),
            &["pages"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[ChildSlot::EachOf {
            array: "pages",
            child: "child",
        }]
    }

    fn allowed_children(&self) -> &'static [&'static str] {
        &["Page"]
    }

    fn usage(&self) -> &'static str {
        "App: {\"component\": \"App\", \"title\", \"pages\": [{\"id\", \"title\", \"child\": \
         \"<a Page's id>\"}] (1 to 24), \"navigation\": \"top|side\", \"initialPage\"}; the home \
         shows it as tabs."
    }

    fn home(&self, app: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let titles: Vec<String> = app
            .get("pages")
            .and_then(Value::as_array)
            .ok_or("it has no pages")?
            .iter()
            .map(|page| {
                page.get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
            .collect();
        if titles.len() != app.children {
            return Err("every page needs its own component".to_string());
        }
        let tabs = HomeNode::Tabs(
            titles
                .into_iter()
                .enumerate()
                .map(|(index, title)| (title, HomeNode::Child(index)))
                .collect(),
        );
        Ok(match heading(3, app.str("title")) {
            Some(title) => HomeNode::Column(vec![title, tabs]),
            None => tabs,
        })
    }
}

/// One page of an app: a title, a description and what it holds.
pub(crate) struct PageComponent;

impl ExtensionComponent for PageComponent {
    fn name(&self) -> &'static str {
        "Page"
    }

    fn schema(&self) -> Value {
        component(
            "Page",
            json!({
                "title": text(),
                "description": text(),
                "layout": one_of(&["stack", "grid", "split"]),
                "children": { "type": "array", "items": component_id(), "maxItems": MAX_HELD }
            }),
            &["children"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[ChildSlot::Many("children")]
    }

    fn allowed_parents(&self) -> &'static [&'static str] {
        &["App"]
    }

    fn usage(&self) -> &'static str {
        "Page: {\"component\": \"Page\", \"title\", \"description\", \"layout\": \
         \"stack|grid|split\", \"children\": [ids]}."
    }

    fn home(&self, page: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let mut nodes: Vec<HomeNode> = heading(3, page.str("title"))
            .into_iter()
            .chain(caption(page.str("description")))
            .collect();
        nodes.extend(held(page));
        Ok(HomeNode::Column(nodes))
    }
}

/// A region of a page: a titled group in a stack, a grid or a split.
pub(crate) struct RegionComponent;

impl ExtensionComponent for RegionComponent {
    fn name(&self) -> &'static str {
        "Region"
    }

    fn schema(&self) -> Value {
        component(
            "Region",
            json!({
                "title": text(),
                "layout": one_of(&["stack", "grid", "split"]),
                "columns": { "type": "integer", "minimum": 1, "maximum": 4 },
                "tone": one_of(TONES),
                "variant": one_of(&["plain", "hero"]),
                "children": { "type": "array", "items": component_id(), "maxItems": MAX_HELD }
            }),
            &["children"],
        )
    }

    fn children(&self) -> &'static [ChildSlot] {
        &[ChildSlot::Many("children")]
    }

    fn usage(&self) -> &'static str {
        "Region: {\"component\": \"Region\", \"title\", \"layout\": \"stack|grid|split\", \
         \"columns\": 1 to 4, \"tone\", \"variant\": \"plain|hero\", \"children\": [ids]}."
    }

    fn home(&self, region: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let mut nodes: Vec<HomeNode> = heading(4, region.str("title")).into_iter().collect();
        nodes.extend(held(region));
        Ok(HomeNode::Card(nodes))
    }
}

/// A 3D scene of primitive shapes with checked labels: no models, textures or actions.
pub(crate) struct SceneComponent;

impl ExtensionComponent for SceneComponent {
    fn name(&self) -> &'static str {
        "Scene3D"
    }

    fn schema(&self) -> Value {
        let colour = json!({ "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" });
        component(
            "Scene3D",
            json!({
                "title": text(),
                "description": text(),
                "height": { "type": "integer", "minimum": 240, "maximum": 960 },
                "background": colour.clone(),
                "camera": {
                    "type": "object",
                    "properties": {
                        "position": vector(-1000.0, 1000.0),
                        "target": vector(-1000.0, 1000.0),
                        "fov": { "type": "number", "minimum": 20, "maximum": 90 }
                    },
                    "additionalProperties": false
                },
                "objects": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": key(),
                            "shape": one_of(&["box", "sphere", "cylinder", "cone", "plane", "torus"]),
                            "position": vector(-1000.0, 1000.0),
                            "rotation": vector(-360.0, 360.0),
                            "scale": vector(0.001, 100.0),
                            "color": colour,
                            "label": text()
                        },
                        "required": ["id", "shape"],
                        "additionalProperties": false
                    },
                    "minItems": 1,
                    "maxItems": MAX_SCENE_OBJECTS
                },
                "autoRotate": { "type": "boolean" },
                "controls": one_of(&["none", "orbit"])
            }),
            &["objects"],
        )
    }

    fn usage(&self) -> &'static str {
        "Scene3D: {\"component\": \"Scene3D\", \"title\", \"description\", \"objects\": [{\"id\", \
         \"shape\": \"box|sphere|cylinder|cone|plane|torus\", \"position\": [x, y, z], \
         \"rotation\", \"scale\", \"color\": \"#rrggbb\", \"label\"}] (1 to 64), \"camera\", \
         \"height\", \"autoRotate\", \"controls\": \"none|orbit\"}; primitives only, no models; the \
         home draws it in 3D in its own colours."
    }

    fn home(&self, spec: &ResolvedComponent<'_>) -> Result<HomeNode, String> {
        let objects = spec
            .get("objects")
            .and_then(Value::as_array)
            .ok_or("it has no objects")?;
        if objects.len() > MAX_SCENE_OBJECTS {
            return Err(format!(
                "it has {} objects; at most {MAX_SCENE_OBJECTS}",
                objects.len()
            ));
        }
        let mut scene = DrawnScene::new(SceneKind::Space);
        for (index, object) in objects.iter().enumerate() {
            let id = object
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("object {index} has no id"))?;
            let shape = object.get("shape").and_then(Value::as_str).unwrap_or("box");
            // A studio plane stands upright in x-y, as three.js draws one: a floor is a plane
            // turned -90 degrees about x. It is drawn as a thin box so it keeps that meaning.
            let unit = if shape == "plane" {
                SceneShape::Box {
                    size: [scene::num(1.0)?, scene::num(1.0)?, scene::num(0.01)?],
                }
            } else {
                scene::unit_shape(shape).ok_or_else(|| format!("{id} has an unknown shape"))?
            };
            let mut drawn = DrawnObject::new(id, unit);
            if let Some(position) = object.get("position") {
                drawn.at = scene::xyz(scene::point(Some(position), "a position")?)?;
            }
            if let Some(rotation) = object.get("rotation") {
                drawn.turn = Some(scene::xyz(scene::point(Some(rotation), "a rotation")?)?);
            }
            if let Some(size) = object.get("scale") {
                drawn.scale = Some(scene::xyz(scene::point(Some(size), "a scale")?)?);
            }
            if let Some(hex) = object.get("color").and_then(Value::as_str) {
                drawn.look.color = scene::nearest_chart(hex);
            }
            drawn.label = object
                .get("label")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .map(scene::short);
            scene.objects.push(drawn);
        }
        if let Some(camera) = spec.get("camera") {
            scene.camera.at = camera
                .get("position")
                .map(|p| scene::point(Some(p), "the camera's position").and_then(scene::xyz))
                .transpose()?;
            scene.camera.target = camera
                .get("target")
                .map(|p| scene::point(Some(p), "the camera's target").and_then(scene::xyz))
                .transpose()?;
            scene.camera.fov = camera
                .get("fov")
                .and_then(Value::as_f64)
                .map(scene::num)
                .transpose()?;
        }
        scene.camera.spin = spec.get("autoRotate") == Some(&Value::Bool(true));
        scene::framed(
            &mut scene,
            spec.get("height").and_then(Value::as_u64),
            spec.str("controls"),
        );
        let body = HomeNode::Scene(Box::new(HomeScene::new(scene)));
        Ok(scene::titled(
            spec.str("title"),
            spec.str("description"),
            body,
        ))
    }
}

/// An application of pages.
///
/// ```rust
/// use adk_ui::studio::App;
///
/// let app = App::new("root").title("Release room").page("overview", "Overview", "overview-page").build();
/// assert_eq!(app["pages"][0]["child"], "overview-page");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct App(Props);

impl App {
    /// An app with no pages yet.
    pub fn new(id: impl Into<String>) -> Self {
        Self(Props::new(id, "App"))
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// A page: its id, its title and the component that draws it (a [`Page`]).
    #[must_use]
    pub fn page(
        mut self,
        id: impl Into<String>,
        title: impl Into<Value>,
        child: impl Into<String>,
    ) -> Self {
        self.0.push(
            "pages",
            json!({ "id": id.into(), "title": title.into(), "child": child.into() }),
        );
        self
    }

    /// Navigation down the side, not across the top.
    #[must_use]
    pub fn side_navigation(mut self) -> Self {
        self.0.set("navigation", "side");
        self
    }
}

component_builder!(App);

/// One page of an app.
#[derive(Debug, Clone, PartialEq)]
pub struct Page(Props);

impl Page {
    /// A page holding `children`, by id.
    pub fn new<I, S>(id: impl Into<String>, children: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut props = Props::new(id, "Page");
        props.set(
            "children",
            children
                .into_iter()
                .map(|child| Value::String(child.into()))
                .collect::<Vec<_>>(),
        );
        Self(props)
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// A line under its title.
    #[must_use]
    pub fn description(mut self, description: impl Into<Value>) -> Self {
        self.0.set("description", description);
        self
    }
}

component_builder!(Page);

/// A titled group on a page.
#[derive(Debug, Clone, PartialEq)]
pub struct Region(Props);

impl Region {
    /// A region holding `children`, by id.
    pub fn new<I, S>(id: impl Into<String>, children: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut props = Props::new(id, "Region");
        props.set(
            "children",
            children
                .into_iter()
                .map(|child| Value::String(child.into()))
                .collect::<Vec<_>>(),
        );
        Self(props)
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// Its children in a grid of `columns` (1 to 4).
    #[must_use]
    pub fn grid(mut self, columns: u8) -> Self {
        self.0.set("layout", "grid");
        self.0.set("columns", columns);
        self
    }
}

component_builder!(Region);

/// A primitive shape of a [`Scene3D`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A box.
    Box,
    /// A sphere.
    Sphere,
    /// A cylinder.
    Cylinder,
    /// A cone.
    Cone,
    /// A flat plane.
    Plane,
    /// A torus.
    Torus,
}

impl Shape {
    fn name(self) -> &'static str {
        match self {
            Self::Box => "box",
            Self::Sphere => "sphere",
            Self::Cylinder => "cylinder",
            Self::Cone => "cone",
            Self::Plane => "plane",
            Self::Torus => "torus",
        }
    }
}

/// One object of a [`Scene3D`].
#[derive(Debug, Clone, PartialEq)]
pub struct SceneObject(Value);

impl SceneObject {
    /// A `shape` at `position`.
    pub fn new(id: impl Into<String>, shape: Shape, position: [f64; 3]) -> Self {
        Self(json!({ "id": id.into(), "shape": shape.name(), "position": position }))
    }

    /// Its label, never a decision's word.
    #[must_use]
    pub fn label(mut self, label: impl Into<Value>) -> Self {
        self.0["label"] = label.into();
        self
    }

    /// Its colour, `#rrggbb`.
    #[must_use]
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.0["color"] = Value::String(color.into());
        self
    }

    /// Its size along each axis.
    #[must_use]
    pub fn scale(mut self, scale: [f64; 3]) -> Self {
        self.0["scale"] = json!(scale);
        self
    }
}

/// A 3D scene of primitive shapes.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene3D(Props);

impl Scene3D {
    /// An empty scene; add objects.
    pub fn new(id: impl Into<String>) -> Self {
        Self(Props::new(id, "Scene3D"))
    }

    /// Its title.
    #[must_use]
    pub fn title(mut self, title: impl Into<Value>) -> Self {
        self.0.set("title", title);
        self
    }

    /// An object. At most sixty-four.
    #[must_use]
    pub fn object(mut self, object: SceneObject) -> Self {
        self.0.push("objects", object.0);
        self
    }

    /// The person may orbit the camera.
    #[must_use]
    pub fn orbit(mut self) -> Self {
        self.0.set("controls", "orbit");
        self
    }
}

component_builder!(Scene3D);
