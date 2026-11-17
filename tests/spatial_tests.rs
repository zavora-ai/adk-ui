//! The spatial catalog checked as `genui` checks it: studio plus 3D charts, scenes, cards with
//! depth and 3D layouts. Rule 5 holds on every text property, found from the schemas,
//! including those inside data lists. Bound data is held to every limit the schema can't see.
//! A pick fills only a choice field the drawing shows; panels and walls hold only what shows.
//! The home gets `genui`'s own scenes.

mod common;

use std::sync::Arc;

use adk_surface::genui::{
    DrawRefusal, ExtensionComponent, HomeNode, MAX_SCENE_TEXT_CHARS, RefusalKind,
    ResolvedComponent, SceneKind, SceneShape, Validator,
};
use adk_ui::catalogs::{self, SPATIAL_ID};
use adk_ui::draft::{Screen, text};
use adk_ui::spatial::{Bar3D, DepthCard, Motion, Object, Scene, Stage};
use serde_json::{Value, json};

fn validator() -> Validator {
    Validator::new(catalogs::spatial())
}

fn component(name: &str) -> Arc<dyn ExtensionComponent> {
    adk_ui::spatial::components()
        .into_iter()
        .find(|component| component.name() == name)
        .unwrap_or_else(|| panic!("no spatial component {name}"))
}

fn screen_of(components: Vec<Value>) -> Screen {
    components
        .into_iter()
        .fold(Screen::new("spatial"), Screen::with)
}

fn refusal(screen: &Screen) -> DrawRefusal {
    match screen.validate(&validator()) {
        Ok(_) => panic!("expected a refusal for {:#}", screen.args()),
        Err(refusal) => refusal,
    }
}

fn home(name: &str, props: Value, children: usize) -> Result<HomeNode, String> {
    component(name).home(&ResolvedComponent {
        id: "x",
        component: name,
        props: &props,
        children,
    })
}

/// The scene a home view draws, below its title.
fn scene_of(view: HomeNode) -> adk_surface::genui::HomeScene {
    match view {
        HomeNode::Scene(scene) => *scene,
        HomeNode::Column(parts) => parts
            .into_iter()
            .find_map(|part| match part {
                HomeNode::Scene(scene) => Some(*scene),
                _ => None,
            })
            .expect("a scene under the title"),
        other => panic!("not a scene: {other:?}"),
    }
}

/// A valid instance of each spatial component, every text property filled.
fn baselines() -> Vec<(&'static str, Vec<Value>)> {
    vec![
        (
            "Bar3D",
            vec![json!({
                "id": "root", "component": "Bar3D", "title": "Sales", "description": "By region",
                "data": [{ "q": "Q1", "north": 3 }], "x": { "key": "q", "label": "Quarter" },
                "series": [{ "key": "north", "label": "North" }], "yLabel": "Revenue"
            })],
        ),
        (
            "Scatter3D",
            vec![json!({
                "id": "root", "component": "Scatter3D", "title": "Sensors", "description": "Field 7",
                "data": [{ "a": 1, "b": 2, "c": 3 }],
                "x": { "key": "a", "label": "East" }, "y": { "key": "b", "label": "Signal" },
                "z": { "key": "c", "label": "North" }
            })],
        ),
        (
            "SurfacePlot",
            vec![json!({
                "id": "root", "component": "SurfacePlot", "title": "Terrain", "description": "Field 7",
                "values": [[1, 2], [3, 4]], "xLabel": "East", "yLabel": "Height", "zLabel": "North"
            })],
        ),
        (
            "Graph3D",
            vec![json!({
                "id": "root", "component": "Graph3D", "title": "Services", "description": "Checkout",
                "nodes": [{ "id": "a", "label": "Gateway" }, { "id": "b" }],
                "edges": [{ "from": "a", "to": "b" }]
            })],
        ),
        (
            "Globe",
            vec![json!({
                "id": "root", "component": "Globe", "title": "Fleet", "description": "In flight",
                "points": [{ "lat": -1.3, "lon": 36.8, "label": "Nairobi" }],
                "arcs": [{ "from": [-1.3, 36.8], "to": [51.5, -0.1], "label": "SH-201" }]
            })],
        ),
        (
            "Scene",
            vec![json!({
                "id": "root", "component": "Scene", "title": "Yard", "description": "Docks",
                "objects": [{ "id": "sign", "shape": "text", "text": "Yard B", "label": "Sign" }]
            })],
        ),
        (
            "DepthCard",
            vec![
                json!({ "id": "root", "component": "DepthCard", "title": "Latency", "front": "held" }),
                text("held", "840 ms"),
            ],
        ),
        (
            "Stage",
            vec![
                json!({ "id": "root", "component": "Stage", "title": "Today", "description": "Figures",
                        "children": ["held"] }),
                text("held", "42 min"),
            ],
        ),
    ]
}

#[test]
fn the_spatial_catalog_is_studio_and_eight_more() {
    let spatial = catalogs::spatial();
    assert_eq!(spatial.id(), SPATIAL_ID);
    assert_eq!(spatial.label(), "spatial 1.0");
    for name in catalogs::studio().components() {
        assert!(spatial.has_component(name), "{name}");
    }
    let added: Vec<String> = adk_ui::spatial::components()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    assert_eq!(
        added,
        [
            "Bar3D",
            "Scatter3D",
            "SurfacePlot",
            "Graph3D",
            "Globe",
            "Scene",
            "DepthCard",
            "Stage"
        ]
    );
    for name in &added {
        assert!(!catalogs::studio().has_component(name), "{name} is new");
        assert!(
            catalogs::HOME_DRAWN.contains(&name.as_str()),
            "the home draws {name}"
        );
    }
}

#[test]
fn every_studio_drawing_is_also_a_spatial_drawing() {
    for fixture in common::fixtures() {
        if fixture.catalog.id() == SPATIAL_ID {
            continue;
        }
        validator()
            .validate(&common::draft(&fixture.screen))
            .unwrap_or_else(|refusal| panic!("{}: {}", fixture.name, refusal.technical()));
    }
}

#[test]
fn rule_5_holds_on_every_text_property_of_every_spatial_component() {
    let cases: [(Value, RefusalKind); 4] = [
        (json!("Allow once"), RefusalKind::LooksLikeDecision),
        (json!("Done"), RefusalKind::LooksLikeDecision),
        (json!("open www.example.com"), RefusalKind::CannotShow),
        (json!("x".repeat(4_001)), RefusalKind::CannotShow),
    ];
    let mut covered = 0;
    for (name, components) in baselines() {
        screen_of(components.clone())
            .validate(&validator())
            .unwrap_or_else(|refusal| panic!("{name}'s baseline: {}", refusal.technical()));
        let mut paths = Vec::new();
        common::text_properties(&component(name).schema(), "", &mut paths);
        for path in paths {
            assert!(
                components[0].pointer(&path).is_some(),
                "{name}'s baseline has no {path}: add it so rule 5 is tested there"
            );
            for (value, kind) in &cases {
                let mut changed = components.clone();
                *changed[0].pointer_mut(&path).unwrap() = value.clone();
                let refused = refusal(&screen_of(changed));
                assert_eq!(refused.kind, *kind, "{name}{path}: {}", refused.technical());
            }
            covered += 1;
        }
    }
    assert!(covered >= 29, "{covered} text properties covered");
}

#[test]
fn bound_data_is_checked_like_literal_data() {
    // A node label in bound data, and one in a literal list.
    let graph =
        |nodes: Value| json!({ "id": "root", "component": "Graph3D", "nodes": nodes, "edges": [] });
    let bound = screen_of(vec![graph(json!({ "path": "/nodes" }))])
        .data(json!({ "nodes": [{ "id": "a", "label": "Allow once" }] }));
    assert_eq!(refusal(&bound).kind, RefusalKind::LooksLikeDecision);
    let literal = screen_of(vec![graph(json!([{ "id": "a", "label": "Approved" }]))]);
    assert_eq!(refusal(&literal).kind, RefusalKind::LooksLikeDecision);

    // Limits the schema can't see through a binding.
    let scatter = screen_of(vec![json!({
        "id": "root", "component": "Scatter3D", "data": { "path": "/rows" },
        "x": { "key": "a" }, "y": { "key": "b" }, "z": { "key": "c" }
    })])
    .data(json!({ "rows": (0..2_001).map(|i| json!({ "a": i, "b": 1, "c": 2 })).collect::<Vec<_>>() }));
    let refused = refusal(&scatter);
    assert!(
        refused.detail.contains("at most 2000"),
        "{}",
        refused.technical()
    );

    let globe = screen_of(vec![
        json!({ "id": "root", "component": "Globe", "points": { "path": "/places" } }),
    ])
    .data(json!({ "places": [{ "lat": 91, "lon": 0 }] }));
    assert!(refusal(&globe).detail.contains("latitude"));

    let edges = screen_of(vec![json!({
        "id": "root", "component": "Graph3D", "nodes": [{ "id": "a" }],
        "edges": { "path": "/edges" }
    })])
    .data(json!({ "edges": [{ "from": "a", "to": "ghost" }] }));
    assert!(refusal(&edges).detail.contains("ghost"));

    let text_value = screen_of(vec![json!({
        "id": "root", "component": "Bar3D", "data": { "path": "/rows" }, "x": { "key": "q" },
        "series": [{ "key": "v" }]
    })])
    .data(json!({ "rows": [{ "q": "Q1", "v": "lots" }] }));
    assert!(refusal(&text_value).detail.contains("not a number"));
}

fn yard(extra: Vec<Value>) -> Vec<Value> {
    let mut components = vec![
        json!({ "id": "root", "component": "Column", "children": ["yard", "dock"] }),
        Scene::new("yard")
            .object(
                Object::new("dock-1", "box", [-2.0, 0.5, 0.0])
                    .label("Dock 1")
                    .state("warning")
                    .value("dock-1"),
            )
            .object(
                Object::new("dock-3", "box", [2.0, 0.5, 0.0])
                    .label("Dock 3")
                    .state("ok")
                    .motion(Motion::pulse(2.0))
                    .value("dock-3"),
            )
            .pick_into("/dock")
            .build(),
        json!({ "id": "dock", "component": "ChoicePicker", "label": "Dock", "value": { "path": "/dock" },
                "options": [{ "label": "Dock 1", "value": "dock-1" }, { "label": "Dock 3", "value": "dock-3" }] }),
    ];
    components.extend(extra);
    components
}

#[test]
fn a_scene_picks_into_a_shown_field_and_holds_together() {
    let shown = screen_of(yard(Vec::new())).data(json!({ "dock": [] }));
    shown
        .validate(&validator())
        .expect("a pick into a shown picker");

    let mut no_picker = yard(Vec::new());
    no_picker.retain(|component| component["id"] != "dock");
    no_picker[0]["children"] = json!(["yard"]);
    assert!(
        refusal(&screen_of(no_picker))
            .detail
            .contains("no choice field")
    );

    let mut unlabelled = yard(Vec::new());
    unlabelled[1]["objects"][0]
        .as_object_mut()
        .unwrap()
        .remove("label");
    unlabelled[1]["objects"][0]
        .as_object_mut()
        .unwrap()
        .remove("state");
    assert!(
        refusal(&screen_of(unlabelled))
            .detail
            .contains("sees what they pick")
    );

    let mut cycle = yard(Vec::new());
    cycle[1]["objects"].as_array_mut().unwrap().extend([
        json!({ "id": "a", "shape": "group", "parent": "b" }),
        json!({ "id": "b", "shape": "group", "parent": "a" }),
    ]);
    assert!(refusal(&screen_of(cycle)).detail.contains("loop"));

    let mut moving = yard(Vec::new());
    moving[1]["objects"] = json!((0..33)
        .map(|i| json!({ "id": format!("o{i}"), "shape": "sphere", "motion": { "kind": "spin" } }))
        .collect::<Vec<_>>());
    moving[1].as_object_mut().unwrap().remove("pickInto");
    assert!(refusal(&screen_of(moving)).detail.contains("more than 32"));

    let mut long = yard(Vec::new());
    long[1]["objects"][0]["label"] = json!("D".repeat(MAX_SCENE_TEXT_CHARS + 30));
    yard_validates_with_a_shortened_label(long);
}

/// A label longer than a scene shows is shortened by the projection, never refused.
fn yard_validates_with_a_shortened_label(components: Vec<Value>) {
    screen_of(components)
        .validate(&validator())
        .expect("a long label is shortened");
}

#[test]
fn panels_and_walls_hold_only_what_shows() {
    let note = text("note", "Free from 14:00");
    let mut paneled = yard(vec![note.clone()]);
    paneled[1]["panels"] = json!([{ "anchor": "dock-3", "child": "note" }]);
    screen_of(paneled.clone())
        .validate(&validator())
        .expect("a panel of text");

    paneled[1]["panels"][0]["anchor"] = json!("crane");
    assert!(refusal(&screen_of(paneled)).detail.contains("crane"));

    let button = vec![
        Stage::new("root", ["go"]).wall().build(),
        json!({ "id": "go", "component": "Button", "child": "go-label", "action": { "event": { "name": "go" } } }),
        text("go-label", "Go"),
    ];
    let screen = screen_of(button).action(
        serde_json::from_value(
            json!({ "id": "go", "label": "Go", "tool": "start_rollback", "args": {} }),
        )
        .unwrap(),
    );
    let refused = refusal(&screen);
    assert!(
        refused.detail.contains("Button 'go'"),
        "{}",
        refused.technical()
    );

    // A carousel turns only when the person turns it; a flip card turns over to its back.
    let carousel = vec![
        Stage::new("root", ["form"]).carousel().build(),
        json!({ "id": "form", "component": "TextField", "label": "Reason", "value": { "path": "/reason" } }),
    ];
    screen_of(carousel)
        .validate(&validator())
        .expect("a carousel may hold a field");
    let flip = vec![
        DepthCard::new("root", "front").flip("back").build(),
        text("front", "840 ms"),
        text("back", "Up from 410 ms"),
    ];
    screen_of(flip).validate(&validator()).expect("a flip card");
    let half = vec![
        json!({ "id": "root", "component": "DepthCard", "effect": "flip", "front": "front" }),
        text("front", "x"),
    ];
    assert!(
        refusal(&screen_of(half))
            .detail
            .contains("front and a back")
    );
}

#[test]
fn the_home_gets_genuis_own_scenes() {
    let bars = scene_of(
        home(
            "Bar3D",
            Bar3D::new(
                "x",
                json!([{ "q": "Q1", "a": 2, "b": 4 }, { "q": "Q2", "a": 3, "b": 8 }]),
                "q",
            )
            .series("a", "North")
            .series("b", "South")
            .title("Sales")
            .build(),
            0,
        )
        .unwrap(),
    );
    assert_eq!(bars.scene.kind, SceneKind::Bars);
    let SceneShape::Bars {
        heights, series, ..
    } = &bars.scene.objects[0].shape
    else {
        panic!("instanced bars first");
    };
    assert_eq!(heights.len(), 4);
    assert_eq!(series, &[0, 1, 0, 1]);
    let values = bars
        .scene
        .values
        .as_ref()
        .expect("heights map back to values");
    let tallest = heights
        .iter()
        .map(|h| h.as_f64().unwrap())
        .fold(0.0, f64::max);
    assert!(
        (values.value(tallest) - 8.0).abs() < 1e-9,
        "the tallest bar stands for 8"
    );
    assert!(bars.text.is_some(), "a text view says each row in words");

    let globe = scene_of(
        home(
            "Globe",
            json!({ "points": [{ "lat": -1.3, "lon": 36.8, "label": "Nairobi" }, { "lat": 51.5, "lon": -0.1 }],
                    "arcs": [{ "from": [-1.3, 36.8], "to": [51.5, -0.1] }], "focus": { "lat": 0, "lon": 30 } }),
            0,
        )
        .unwrap(),
    );
    assert_eq!(globe.scene.kind, SceneKind::Globe);
    assert!(matches!(
        globe.scene.objects[0].shape,
        SceneShape::Globe { .. }
    ));
    assert!(
        globe
            .scene
            .objects
            .iter()
            .any(|o| matches!(o.shape, SceneShape::Line { .. })),
        "a route"
    );
    assert!(globe.scene.camera.at.is_some(), "seen from above its focus");

    let card = home("DepthCard", DepthCard::new("x", "f").flip("b").build(), 2).unwrap();
    assert!(matches!(card, HomeNode::Depth { ref children, .. } if children.len() == 2));
}

/// What Claude drew for "an animated solar system" in a live run: Uranus's orbit is slower than
/// a scene moves. The refusal names the object and the rule, not the whole list, so the agent
/// can fix it.
#[test]
fn a_refused_scene_says_which_object_broke_which_rule() {
    let planet = |id: &str, seconds: f64| json!({ "id": id, "shape": "sphere", "label": id, "motion": { "kind": "orbit", "seconds": seconds } });
    let solar = screen_of(vec![json!({
        "id": "root", "component": "Scene",
        "objects": [planet("Earth", 4.0), planet("Saturn", 117.8), planet("Uranus", 336.0)]
    })]);
    let refused = refusal(&solar);
    assert_eq!(refused.kind, RefusalKind::CannotShow);
    assert!(
        refused.detail.contains("/objects/2/motion/seconds"),
        "{}",
        refused.technical()
    );
    assert!(
        refused.detail.contains("maximum of 120"),
        "{}",
        refused.technical()
    );
    assert!(
        !refused.detail.contains("Saturn"),
        "{}",
        refused.technical()
    );
}
