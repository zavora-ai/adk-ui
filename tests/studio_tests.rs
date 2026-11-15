//! The studio catalog checked as `genui` checks it: apps, pages, regions and primitive scenes
//! on top of insight, rule 5 on every text property (found from the schemas), containers that
//! keep every forms check for what they hold, a scene's limits, what the home shows, and the
//! approved-kit bridge to a custom host's tokens.

use std::sync::Arc;

use adk_surface::genui::{
    Catalog, DrawRefusal, ExtensionComponent, HomeNode, RefusalKind, ResolvedComponent, Validator,
};
use adk_ui::catalogs::{self, STUDIO_ID};
use adk_ui::draft::{Screen, text};
use adk_ui::insight::Metric;
use adk_ui::kit::{ApprovedKit, BrandKitManifest, KitGenerator, KitSpec, KitStatus};
use adk_ui::studio::{App, Page, Region, Scene3D, SceneObject, Shape};
use serde_json::{Value, json};

fn studio() -> &'static Catalog {
    catalogs::studio()
}

fn component(name: &str) -> Arc<dyn ExtensionComponent> {
    adk_ui::studio::components()
        .into_iter()
        .find(|component| component.name() == name)
        .unwrap_or_else(|| panic!("no studio component {name}"))
}

fn refusal(screen: &Screen) -> DrawRefusal {
    match screen.validate(&Validator::new(studio())) {
        Ok(_) => panic!("expected a refusal for {:#}", screen.args()),
        Err(refusal) => refusal,
    }
}

/// A valid instance of each studio component with what it holds.
fn baselines() -> Vec<(&'static str, Vec<Value>)> {
    vec![
        (
            "App",
            vec![
                json!({ "id": "root", "component": "App", "title": "Release room",
                        "pages": [{ "id": "overview", "title": "Overview", "child": "held" }] }),
                text("held", "Ready"),
            ],
        ),
        (
            "Page",
            vec![
                json!({ "id": "root", "component": "Page", "title": "Overview",
                        "description": "This week", "children": ["held"] }),
                text("held", "Ready"),
            ],
        ),
        (
            "Region",
            vec![
                json!({ "id": "root", "component": "Region", "title": "Health", "children": ["held"] }),
                text("held", "Ready"),
            ],
        ),
        (
            "Scene3D",
            vec![json!({
                "id": "root", "component": "Scene3D", "title": "Warehouse", "description": "Bay 4",
                "objects": [{ "id": "dock", "shape": "box", "position": [0, 0, 0], "label": "Dock" }]
            })],
        ),
    ]
}

fn text_properties(schema: &Value, at: &str, out: &mut Vec<String>) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };
    for (name, property) in properties {
        let here = format!("{at}/{name}");
        if property
            .get("$ref")
            .and_then(Value::as_str)
            .is_some_and(|r| r.ends_with("/DynamicString"))
        {
            out.push(here);
        } else if property.get("type") == Some(&json!("array")) {
            if let Some(items) = property.get("items") {
                text_properties(items, &format!("{here}/0"), out);
            }
        } else if property.get("type") == Some(&json!("object")) {
            text_properties(property, &here, out);
        }
    }
}

fn screen_of(components: Vec<Value>) -> Screen {
    components
        .into_iter()
        .fold(Screen::new("studio"), Screen::with)
}

#[test]
fn the_studio_catalog_is_insight_and_four_more() {
    assert_eq!(studio().id(), STUDIO_ID);
    assert_eq!(studio().label(), "studio 1.0");
    for name in catalogs::insight().components() {
        assert!(studio().has_component(name), "{name}");
    }
    for name in ["App", "Page", "Region", "Scene3D"] {
        assert!(
            studio().has_component(name) && !catalogs::insight().has_component(name),
            "{name}"
        );
    }
}

#[test]
fn rule_5_holds_on_every_text_property_of_every_studio_component() {
    let cases: [(Value, RefusalKind); 4] = [
        (json!("Allow once"), RefusalKind::LooksLikeDecision),
        (json!("Done"), RefusalKind::LooksLikeDecision),
        (json!("open www.example.com"), RefusalKind::CannotShow),
        (json!("x".repeat(4_001)), RefusalKind::CannotShow),
    ];
    let mut covered = 0;
    for (name, components) in baselines() {
        assert!(
            screen_of(components.clone())
                .validate(&Validator::new(studio()))
                .is_ok(),
            "{name}'s baseline is accepted"
        );
        let schema = component(name).schema();
        let own = schema["allOf"]
            .as_array()
            .and_then(|parts| parts.iter().find(|part| part.get("properties").is_some()))
            .unwrap();
        let mut paths = Vec::new();
        text_properties(own, "", &mut paths);
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
    assert!(covered >= 7, "{covered} text properties covered");
}

#[test]
fn a_container_keeps_every_forms_check_for_what_it_holds() {
    let secret = refusal(&screen_of(vec![
        Page::new("root", ["pin"]).title("Sign in").build(),
        json!({ "id": "pin", "component": "TextField", "label": "PIN", "value": { "path": "/pin" } }),
    ]));
    assert_eq!(secret.kind, RefusalKind::AsksForSecret);
    let looped = refusal(&screen_of(vec![
        Region::new("root", ["inner"]).build(),
        Region::new("inner", ["root"]).build(),
    ]));
    assert!(looped.detail.contains("loops"), "{}", looped.technical());
    let missing = refusal(&screen_of(vec![
        App::new("root").page("p", "P", "nowhere").build(),
    ]));
    assert!(
        missing.detail.contains("does not exist"),
        "{}",
        missing.technical()
    );
}

#[test]
fn a_scene_takes_primitives_and_checked_colours_only() {
    let scene = |object: Value| {
        screen_of(vec![
            json!({ "id": "root", "component": "Scene3D", "objects": [object] }),
        ])
    };
    for bad in [
        json!({ "id": "a", "shape": "box", "color": "red" }),
        json!({ "id": "a", "shape": "box", "color": "url(javascript:x)" }),
        json!({ "id": "a", "shape": "teapot" }),
        json!({ "id": "a", "shape": "box", "assetId": "logo" }),
        json!({ "id": "a", "shape": "box", "action": { "event": { "name": "go" } } }),
        json!({ "id": "a", "shape": "box", "scale": [0, 1, 1] }),
    ] {
        assert_eq!(
            refusal(&scene(bad.clone())).kind,
            RefusalKind::CannotShow,
            "{bad}"
        );
    }
    let mut built = Scene3D::new("root").title("Bays").orbit();
    for i in 0..65 {
        built = built.object(SceneObject::new(
            format!("o{i}"),
            Shape::Box,
            [f64::from(i), 0.0, 0.0],
        ));
    }
    assert_eq!(
        refusal(&screen_of(vec![built.build()])).kind,
        RefusalKind::CannotShow
    );
    let fine = Scene3D::new("root")
        .object(
            SceneObject::new("dock", Shape::Box, [0.0, 0.0, 0.0])
                .label("Dock")
                .color("#2563eb"),
        )
        .build();
    assert!(
        screen_of(vec![fine])
            .validate(&Validator::new(studio()))
            .is_ok()
    );
}

fn home(name: &str, props: Value, children: usize) -> HomeNode {
    let resolved = ResolvedComponent {
        id: "c",
        component: name,
        props: &props,
        children,
    };
    component(name)
        .home(&resolved)
        .unwrap_or_else(|why| panic!("{name}: {why}"))
}

#[test]
fn the_home_shows_apps_as_tabs_and_scenes_as_what_they_hold() {
    assert_eq!(
        home(
            "App",
            json!({ "pages": [{ "id": "a", "title": "Overview", "child": "x" }, { "id": "b", "title": "Risks", "child": "y" }] }),
            2
        ),
        HomeNode::Tabs(vec![
            ("Overview".into(), HomeNode::Child(0)),
            ("Risks".into(), HomeNode::Child(1)),
        ])
    );
    assert_eq!(
        home(
            "Page",
            json!({ "title": "Overview", "description": "This week", "children": ["x"] }),
            1
        ),
        HomeNode::Column(vec![
            HomeNode::heading(3, "Overview"),
            HomeNode::caption("This week"),
            HomeNode::Child(0)
        ])
    );
    assert_eq!(
        home("Region", json!({ "children": ["x", "y"] }), 2),
        HomeNode::Card(vec![HomeNode::Child(0), HomeNode::Child(1)])
    );
    let objects: Vec<Value> = (0..30)
        .map(|i| json!({ "id": format!("o{i}"), "shape": "sphere", "label": format!("Crate {i}") }))
        .collect();
    let HomeNode::Column(parts) = home("Scene3D", json!({ "title": "Bay", "objects": objects }), 0)
    else {
        panic!("a scene is a column");
    };
    assert_eq!(parts[0], HomeNode::heading(4, "Bay"));
    assert_eq!(parts[1], HomeNode::text("Crate 0 · sphere"));
    assert_eq!(parts.last(), Some(&HomeNode::caption("and 6 more shapes")));
    assert!(parts.len() <= 64, "within genui's home view limit");
    assert_eq!(
        home(
            "Scene3D",
            json!({ "objects": [{ "id": "a", "shape": "box" }] }),
            0
        ),
        HomeNode::Column(vec![HomeNode::caption("A 3D scene of 1 shape")])
    );
}

#[test]
fn an_app_of_pages_with_insight_inside_is_accepted() {
    let screen = Screen::new("release")
        .data(json!({ "open": 3 }))
        .with(
            App::new("root")
                .title("Release room")
                .page("now", "Now", "now-page"),
        )
        .with(Page::new("now-page", ["health"]).title("Right now"))
        .with(Region::new("health", ["open"]).title("Health").grid(2))
        .with(Metric::new("open", "Open incidents", adk_ui::bind("/open")));
    assert!(screen.validate(&Validator::new(studio())).is_ok());
    assert!(
        screen
            .validate(&Validator::new(catalogs::insight()))
            .is_err(),
        "insight has no App"
    );
}

fn approved() -> BrandKitManifest {
    let spec: KitSpec = serde_json::from_value(json!({
        "name": "Fintech Pro",
        "version": "0.1.0",
        "brand": { "vibe": "trustworthy" },
        "colors": { "primary": "#2F6BFF" },
        "typography": { "family": "Source Sans 3" }
    }))
    .unwrap();
    let mut manifest = KitGenerator::new().generate(&spec).manifest;
    manifest.status = KitStatus::Approved;
    manifest.provenance.reviewed_by = Some("brand-team".into());
    manifest.provenance.approved_at = Some("2026-11-16".into());
    manifest
}

#[test]
fn only_an_approved_kit_restyles_a_custom_host_and_never_a_decision() {
    let mut draft = approved();
    draft.status = KitStatus::Draft;
    assert!(ApprovedKit::try_from(draft).is_err());
    let mut unsigned = approved();
    unsigned.provenance.reviewed_by = None;
    assert!(ApprovedKit::try_from(unsigned).is_err());

    let mut poisoned = approved();
    poisoned.tokens.colors.chart = vec!["url(javascript:x)".into()];
    let refused = ApprovedKit::try_from(poisoned).unwrap_err().to_string();
    assert!(
        refused.contains("chart"),
        "a kit whose colour is not a colour never applies: {refused}"
    );

    let mut manifest = approved();
    manifest.tokens.colors.chart = vec!["#123456".into()];
    let kit = ApprovedKit::try_from(manifest).expect("an approved kit applies");
    let css = kit.surface_css();
    assert!(
        css.starts_with("[data-adk-ui-kit=\"zavora.ai:adk-ui/kit/fintech-pro@0.1.0\"] {"),
        "{css}"
    );
    assert!(css.contains("--accent: #2F6BFF;"), "{css}");
    assert!(css.contains("--chart-1: #123456;"), "{css}");
    assert!(
        !css.contains("javascript"),
        "a value that is not a colour is left out"
    );
    assert!(
        !css.contains("--decision") && !css.contains("--pill"),
        "a kit never restyles a decision"
    );
    assert!(css.contains("--font: \"Source Sans 3\""), "{css}");
    assert_eq!(kit.a2ui_theme(), json!({ "primaryColor": "#2F6BFF" }));
}
