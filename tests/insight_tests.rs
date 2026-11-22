//! The insight catalog checked as `genui` checks it: a superset of `forms`, every builder's
//! output accepted, rule 5 on every text property (found from the schemas, so a new one is
//! covered or the test fails), the data the components bind, their limits, and what the home
//! shows for each.

use std::sync::Arc;

use adk_surface::genui::{
    Catalog, ChartAxis, ChartKind as HomeChartKind, DrawRefusal, ExtensionComponent, HomeNode,
    RefusalKind, ResolvedComponent, Validator,
};
use adk_ui::catalogs::{self, INSIGHT_ID};
use adk_ui::draft::{Screen, column};
use adk_ui::insight::{
    Badge, Chart, ChartKind, KeyValue, Metric, Progress, Sparkline, Tone, VegaLite, XAxis, bind,
};
use serde_json::{Value, json};

fn insight() -> &'static Catalog {
    catalogs::insight()
}

fn validator() -> Validator {
    Validator::new(insight())
}

fn component(name: &str) -> Arc<dyn ExtensionComponent> {
    adk_ui::insight::components()
        .into_iter()
        .find(|component| component.name() == name)
        .unwrap_or_else(|| panic!("no insight component {name}"))
}

/// A screen whose root is `root`, over `model`.
fn screen(root: Value, model: Value) -> Screen {
    let mut root = root;
    root["id"] = json!("root");
    Screen::new("insight").data(model).with(root)
}

fn refusal(screen: &Screen) -> DrawRefusal {
    match screen.validate(&validator()) {
        Ok(_) => panic!("expected a refusal for {:#}", screen.args()),
        Err(refusal) => refusal,
    }
}

/// A valid instance of every insight component, with the data model it binds.
fn baselines() -> Vec<(&'static str, Value, Value)> {
    vec![
        (
            "Chart",
            json!({
                "component": "Chart", "kind": "bar", "title": "Revenue",
                "data": { "path": "/rows" },
                "x": { "key": "month", "type": "category", "label": "Month" },
                "series": [{ "key": "revenue", "label": "Revenue" }],
                "yLabel": "USD"
            }),
            json!({ "rows": [{ "month": "Jan", "revenue": 4 }, { "month": "Feb", "revenue": 7 }] }),
        ),
        (
            "Sparkline",
            json!({ "component": "Sparkline", "data": [1, 3, 2], "label": "Latency" }),
            json!({}),
        ),
        (
            "Metric",
            json!({
                "component": "Metric", "label": "Open incidents", "value": 3, "unit": "today",
                "caption": "Down from 5", "delta": { "value": -2, "goodWhen": "down" }
            }),
            json!({}),
        ),
        (
            "KeyValue",
            json!({
                "component": "KeyValue", "title": "Shipment",
                "items": [{ "key": "Carrier", "value": "DHL" }]
            }),
            json!({}),
        ),
        (
            "Progress",
            json!({ "component": "Progress", "label": "Upload", "value": 40 }),
            json!({}),
        ),
        (
            "Badge",
            json!({ "component": "Badge", "text": "Shipped", "tone": "success" }),
            json!({}),
        ),
        (
            "VegaLite",
            json!({
                "component": "VegaLite", "title": "Latency",
                "spec": {
                    "mark": "line",
                    "encoding": { "x": { "field": "day", "type": "ordinal" },
                                  "y": { "field": "ms", "type": "quantitative" } }
                },
                "data": { "path": "/rows" }
            }),
            json!({ "rows": [{ "day": "Mon", "ms": 120 }, { "day": "Tue", "ms": 98 }] }),
        ),
    ]
}

/// Every JSON pointer, within a component instance, of a property the schema types as text.
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

fn own_properties(schema: &Value) -> &Value {
    schema["allOf"]
        .as_array()
        .and_then(|parts| parts.iter().find(|part| part.get("properties").is_some()))
        .expect("an insight schema declares its properties in allOf")
}

#[test]
fn the_insight_catalog_is_forms_and_its_seven_components() {
    assert_eq!(insight().id(), INSIGHT_ID);
    assert_eq!(insight().label(), "insight 1.0");
    assert!(
        Catalog::forms()
            .components()
            .all(|name| insight().has_component(name))
    );
    let mut added: Vec<&str> = insight().extensions().collect();
    added.sort_unstable();
    assert_eq!(
        added,
        [
            "Badge",
            "Chart",
            "KeyValue",
            "Metric",
            "Progress",
            "Sparkline",
            "VegaLite"
        ]
    );
    assert_eq!(catalogs::all().len(), 3, "insight, studio and spatial");

    let tools = adk_ui::toolset().tools();
    let names: Vec<&str> = tools.iter().map(|tool| tool.name()).collect();
    assert_eq!(
        names,
        adk_core::DRAW_TOOL_NAMES,
        "adk-ui adds no render tool name"
    );
    let render_screen = tools
        .iter()
        .find(|tool| tool.name() == "render_screen")
        .unwrap();
    for name in added {
        assert!(
            render_screen.description().contains(&format!("{name}: ")),
            "{name}"
        );
        assert!(
            adk_ui::prompts::instructions(insight()).contains(name),
            "{name}"
        );
    }
}

#[test]
fn every_builders_output_is_accepted() {
    let screen = Screen::new("ops")
        .title("Operations")
        .data(json!({
            "sales": [{ "month": "Jan", "revenue": 4200, "cost": 3100 }, { "month": "Feb", "revenue": 5100, "cost": 3300 }],
            "latency": [120, 98, 143, 110],
            "open": 3,
            "upload": 40
        }))
        .with(column("root", ["sales", "latency", "open", "shipment", "upload", "state"]))
        .with(
            Chart::new("sales", ChartKind::Bar)
                .title("Revenue and cost")
                .data(bind("/sales"))
                .x(XAxis::category("month").label("Month"))
                .series("revenue", "Revenue")
                .series("cost", "Cost")
                .currency("USD"),
        )
        .with(Sparkline::new("latency", bind("/latency")).label("Latency").tone(Tone::Info))
        .with(Metric::new("open", "Open incidents", bind("/open")).delta(-2.0, false).status("warning"))
        .with(KeyValue::new("shipment").title("Shipment").pair("Carrier", "DHL").pair("Weight", 12))
        .with(Progress::new("upload", "Upload", bind("/upload")).tone(Tone::Neutral))
        .with(Badge::new("state", "Shipped", Tone::Success));
    let validated = screen
        .validate(&validator())
        .expect("every builder's output is accepted");
    let shown: Vec<&str> = validated
        .texts
        .iter()
        .map(|text| text.text.as_str())
        .collect();
    for text in [
        "Revenue and cost",
        "Latency",
        "Open incidents",
        "Carrier",
        "Upload",
        "Shipped",
    ] {
        assert!(shown.contains(&text), "{text} in {shown:?}");
    }
}

/// Rule 5 on every text property every insight schema declares: a decision's phrase, a
/// decision's word, a disguised phrase, a link and an overlong string are refused, typed in
/// or bound.
#[test]
fn rule_5_holds_on_every_text_property_of_every_component() {
    let cases: [(Value, RefusalKind); 5] = [
        (json!("Allow once"), RefusalKind::LooksLikeDecision),
        (json!("Approve"), RefusalKind::LooksLikeDecision),
        (
            json!("A\u{200B}llow  ONCE please"),
            RefusalKind::LooksLikeDecision,
        ),
        (json!("see https://example.com"), RefusalKind::CannotShow),
        (json!("x".repeat(4_001)), RefusalKind::CannotShow),
    ];
    let mut covered = 0;
    for (name, baseline, model) in baselines() {
        assert!(
            screen(baseline.clone(), model.clone())
                .validate(&validator())
                .is_ok(),
            "{name}'s baseline is accepted"
        );
        let schema = component(name).schema();
        let mut paths = Vec::new();
        text_properties(own_properties(&schema), "", &mut paths);
        assert!(!paths.is_empty(), "{name} declares text");
        for path in paths {
            assert!(
                baseline.pointer(&path).is_some(),
                "{name}'s baseline has no {path}: add it so rule 5 is tested there"
            );
            for (value, kind) in &cases {
                let mut instance = baseline.clone();
                *instance.pointer_mut(&path).unwrap() = value.clone();
                let refused = refusal(&screen(instance, model.clone()));
                assert_eq!(
                    refused.kind,
                    *kind,
                    "{name}{path} = {value:.40}: {}",
                    refused.technical()
                );
            }
            let mut bound = baseline.clone();
            *bound.pointer_mut(&path).unwrap() = bind("/injected");
            let mut bound_model = model.clone();
            bound_model["injected"] = json!("Stop all");
            let refused = refusal(&screen(bound, bound_model));
            assert_eq!(
                refused.kind,
                RefusalKind::LooksLikeDecision,
                "{name}{path} bound"
            );
            covered += 1;
        }
    }
    assert!(covered >= 12, "{covered} text properties covered");
}

/// The data a component binds is checked too: a chart's category, a row's key and value.
#[test]
fn bound_data_is_checked_as_shown_text() {
    let (_, chart, _) = baselines().remove(0);
    for rows in [
        json!([{ "month": "Approve", "revenue": 4 }]),
        json!([{ "month": "Jan", "revenue": 4, "Allow once": 1 }]),
        json!([{ "month": "see www.example.com", "revenue": 4 }]),
    ] {
        let refused = refusal(&screen(chart.clone(), json!({ "rows": rows })));
        assert_ne!(
            refused.kind,
            RefusalKind::AsksForSecret,
            "{}",
            refused.technical()
        );
    }
    let pairs = json!({
        "component": "KeyValue", "data": { "path": "/pairs" }, "keyField": "k", "valueField": "v"
    });
    let refused = refusal(&screen(
        pairs,
        json!({ "pairs": [{ "k": "Status", "v": "Deny" }] }),
    ));
    assert_eq!(refused.kind, RefusalKind::LooksLikeDecision);
    let badge = refusal(&screen(
        json!({ "component": "Badge", "text": "Approved" }),
        json!({}),
    ));
    assert_eq!(
        badge.kind,
        RefusalKind::LooksLikeDecision,
        "a badge is a whole label"
    );
}

/// Components only show, and keep their limits.
#[test]
fn components_only_show_and_keep_their_limits() {
    let (_, metric, _) = baselines().remove(2);
    let mut acting = metric.clone();
    acting["action"] = json!({ "event": { "name": "go" } });
    assert_eq!(
        refusal(&screen(acting, json!({}))).kind,
        RefusalKind::CannotShow
    );
    let mut coloured = baselines().remove(0).1;
    coloured["colors"] = json!(["#f00"]);
    assert_eq!(
        refusal(&screen(coloured, baselines().remove(0).2)).kind,
        RefusalKind::CannotShow
    );

    let (_, chart, _) = baselines().remove(0);
    let rows: Vec<Value> = (0..501)
        .map(|i| json!({ "month": format!("M{i}"), "revenue": i }))
        .collect();
    let refused = refusal(&screen(chart.clone(), json!({ "rows": rows })));
    assert_eq!(
        refused.kind,
        RefusalKind::CannotShow,
        "{}",
        refused.technical()
    );
    let mut wide = chart.clone();
    wide["series"] = json!(
        (0..9)
            .map(|i| json!({ "key": format!("s{i}") }))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        refusal(&screen(wide, json!({ "rows": [] }))).kind,
        RefusalKind::CannotShow
    );
    let mut pie = chart;
    pie["kind"] = json!("pie");
    let negative =
        json!({ "rows": [{ "month": "A", "revenue": 3 }, { "month": "B", "revenue": -1 }] });
    assert!(refusal(&screen(pie, negative)).detail.contains("negative"));
    let many: Vec<Value> = (0..51)
        .map(|i| json!({ "key": format!("K{i}"), "value": i }))
        .collect();
    let crowded = json!({ "component": "KeyValue", "items": many });
    assert_eq!(
        refusal(&screen(crowded, json!({}))).kind,
        RefusalKind::CannotShow
    );
}

fn home(name: &str, props: Value) -> HomeNode {
    let component = component(name);
    let resolved = ResolvedComponent {
        id: "c",
        component: name,
        props: &props,
        children: 0,
    };
    component
        .home(&resolved)
        .unwrap_or_else(|why| panic!("{name}: {why}"))
}

fn chart_of(node: &HomeNode) -> &adk_surface::genui::DrawnChart {
    match node {
        HomeNode::Chart(chart) => chart,
        HomeNode::Column(nodes) | HomeNode::Card(nodes) => nodes
            .iter()
            .find_map(|node| match node {
                HomeNode::Chart(chart) => Some(chart),
                _ => None,
            })
            .expect("a chart"),
        other => panic!("no chart in {other:?}"),
    }
}

#[test]
fn the_home_shows_each_component_in_forms_and_charts() {
    let chart = home(
        "Chart",
        json!({
            "kind": "line", "title": "Revenue",
            "data": [{ "month": "Jan", "revenue": 4 }, { "month": "Feb", "revenue": 7 }],
            "x": { "key": "month" }, "series": [{ "key": "revenue", "label": "Revenue" }]
        }),
    );
    let drawn = chart_of(&chart);
    assert_eq!(drawn.kind, HomeChartKind::Line);
    assert_eq!(
        drawn.x,
        ChartAxis::Category {
            labels: vec!["Jan".into(), "Feb".into()]
        }
    );
    assert_eq!(drawn.series[0].name, "Revenue");
    assert!(
        matches!(&chart, HomeNode::Column(nodes) if nodes[0] == HomeNode::heading(4, "Revenue"))
    );

    let timed = home(
        "Chart",
        json!({
            "kind": "area",
            "data": [{ "t": "2026-11-16T09:00:00Z", "v": 1 }, { "t": "2026-11-16T15:00:00Z", "v": 4 }],
            "x": { "key": "t", "type": "time" }, "series": [{ "key": "v" }], "window": 2
        }),
    );
    let drawn = chart_of(&timed);
    assert_eq!(drawn.x, ChartAxis::Time);
    assert_eq!(
        drawn.ticks.first().map(|tick| tick.text.as_str()),
        Some("09:00")
    );
    assert_eq!(
        drawn.series[0].name, "v",
        "a series without a label is named by its key"
    );

    let metric = home(
        "Metric",
        json!({
            "label": "Open incidents", "value": 4200, "format": "currency", "currency": "USD",
            "delta": { "value": -2, "goodWhen": "down" }, "status": "critical", "trend": [5, 4, 3]
        }),
    );
    let HomeNode::Card(parts) = &metric else {
        panic!("a metric is a card: {metric:?}")
    };
    assert_eq!(parts[0], HomeNode::caption("Open incidents"));
    assert_eq!(parts[1], HomeNode::heading(2, "USD 4,200"));
    assert_eq!(parts[2], HomeNode::text("-2 · better"));
    assert!(parts.contains(&HomeNode::caption("Critical")));
    assert_eq!(chart_of(&metric).kind, HomeChartKind::Sparkline);

    assert_eq!(
        home(
            "KeyValue",
            json!({ "items": [{ "key": "Carrier", "value": "DHL" }] })
        ),
        HomeNode::Column(vec![HomeNode::Row(vec![
            HomeNode::caption("Carrier"),
            HomeNode::text("DHL")
        ])])
    );
    assert_eq!(
        home("Progress", json!({ "label": "Upload", "value": 140 })),
        HomeNode::text("Upload · 100%")
    );
    assert_eq!(
        home(
            "Progress",
            json!({ "label": "Indexing", "indeterminate": true })
        ),
        HomeNode::text("Indexing · in progress")
    );
    assert_eq!(
        home("Badge", json!({ "text": "Shipped" })),
        HomeNode::caption("Shipped")
    );
    assert_eq!(
        chart_of(&home("Sparkline", json!({ "data": [1, 2, 3] }))).series[0]
            .y
            .len(),
        3
    );
}

#[test]
fn a_projection_says_what_is_wrong_with_its_data() {
    let component = component("Chart");
    let props = json!({
        "kind": "bar", "data": [{ "month": "Jan", "revenue": "lots" }],
        "x": { "key": "month" }, "series": [{ "key": "revenue" }]
    });
    let resolved = ResolvedComponent {
        id: "c",
        component: "Chart",
        props: &props,
        children: 0,
    };
    let why = component.home(&resolved).unwrap_err();
    assert!(why.contains("row 0's revenue is not a number"), "{why}");
    // The validator turns it into a refusal the agent reads.
    let refused = refusal(&screen(
        json!({ "component": "Chart", "kind": "bar",
        "data": [{ "month": "Jan", "revenue": "lots" }], "x": { "key": "month" },
        "series": [{ "key": "revenue" }] }),
        json!({}),
    ));
    assert!(
        refused.detail.contains("not a number"),
        "{}",
        refused.technical()
    );
}

/// A Vega-Lite spec is drawn as the home's own chart; what the subset does not read is refused
/// by name, and every string in the spec is checked as shown text.
#[test]
fn a_vega_lite_spec_is_drawn_as_the_homes_chart() {
    let spec = json!({
        "mark": "bar",
        "encoding": {
            "x": { "field": "quarter", "type": "nominal", "title": "Quarter" },
            "y": { "field": "revenue", "type": "quantitative", "title": "Revenue" },
            "color": { "field": "region", "type": "nominal" }
        }
    });
    let rows = json!([
        { "quarter": "Q1", "region": "East", "revenue": 4 },
        { "quarter": "Q1", "region": "West", "revenue": 3 },
        { "quarter": "Q2", "region": "East", "revenue": 6 }
    ]);
    let built = Screen::new("sales")
        .data(json!({ "rows": rows }))
        .with(column("root", ["chart"]))
        .with(
            VegaLite::new("chart", spec.clone())
                .title("Revenue by region")
                .data(bind("/rows")),
        );
    built
        .validate(&validator())
        .expect("a VegaLite chart is accepted");

    let drawn = home(
        "VegaLite",
        json!({ "title": "Revenue by region", "spec": spec.clone(), "data": rows }),
    );
    let chart = chart_of(&drawn);
    assert_eq!(chart.kind, HomeChartKind::Bar);
    assert_eq!(
        chart.x,
        ChartAxis::Category {
            labels: vec!["Q1".into(), "Q2".into()]
        }
    );
    let names: Vec<&str> = chart.series.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["East", "West"], "a series per color value");

    for (bad, says) in [
        (json!({ "transform": [{ "filter": "true" }] }), "transforms"),
        (json!({ "layer": [] }), "layers"),
        (json!({ "params": [] }), "params"),
        (json!({ "mark": "rect" }), "not drawn here"),
    ] {
        let mut wrong = spec.clone();
        for (key, value) in bad.as_object().unwrap() {
            wrong[key] = value.clone();
        }
        let refused = refusal(&screen(
            json!({ "component": "VegaLite", "spec": wrong, "data": { "path": "/rows" } }),
            json!({ "rows": [] }),
        ));
        assert!(
            refused.detail.contains(says),
            "{says}: {}",
            refused.technical()
        );
    }

    for (pointer, value, kind) in [
        (
            "/encoding/y/title",
            json!("Allow once"),
            RefusalKind::LooksLikeDecision,
        ),
        ("/title", json!("Approve"), RefusalKind::LooksLikeDecision),
        (
            "/data",
            json!({ "url": "https://example.com/rows.csv" }),
            RefusalKind::CannotShow,
        ),
    ] {
        let mut wrong = spec.clone();
        if pointer == "/data" || pointer == "/title" {
            wrong[&pointer[1..]] = value.clone();
        } else {
            *wrong.pointer_mut(pointer).unwrap() = value.clone();
        }
        let refused = refusal(&screen(
            json!({ "component": "VegaLite", "spec": wrong, "data": { "path": "/rows" } }),
            json!({ "rows": [{ "quarter": "Q1", "region": "East", "revenue": 4 }] }),
        ));
        assert_eq!(refused.kind, kind, "{pointer}: {}", refused.technical());
    }
}
