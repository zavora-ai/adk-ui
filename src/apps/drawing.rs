//! `show_drawing`: a drawing in adk-ui's catalogs shown read-only as an MCP App, for hosts
//! other than the ADK-Rust home (any client that frames MCP Apps 2026-01-26).
//!
//! The drawing is checked as genui checks every drawing (rule 5: no decision's words, secrets
//! or links, every string and bound value included) and resolved on the server with
//! [`drawn_tree`]: the page only sets text from the tree it is given. It declares no actions,
//! so a button is refused before anything is shown; sending stays in the home, where the person
//! decides on a card the home draws. A host without MCP Apps shows the call's text: the
//! drawing's words, line by line.

use std::fmt::Write as _;

use adk_surface::genui::{
    Catalog, DrawnKind, DrawnNode, SurfaceDraft, Validator, components_in, drawn_tree,
};
use rmcp::model::{
    CallToolResult, ContentBlock, JsonObject, ResourceContents, Tool, ToolAnnotations,
};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{APP_MIME_TYPE, meta};
use crate::catalogs;

/// The tool's name.
pub const DRAWING_TOOL: &str = "show_drawing";
/// The page's resource.
pub const DRAWING_URI: &str = "ui://adk-ui/drawing";
/// The page: a read-only renderer of the resolved tree, reaching no site.
pub const DRAWING_PAGE: &str = include_str!("drawing_app.html");
/// The most characters of the text a host without MCP Apps shows.
const TEXT_CHARS: usize = 4_000;

/// A `show_drawing` call's arguments.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DrawingInput {
    /// The title above the drawing.
    #[serde(default)]
    pub title: Option<String>,
    /// A2UI components, in 1.0's syntax or v0.9.1's, one with id `root`.
    pub components: Vec<Value>,
    /// The data model the components bind.
    #[serde(default)]
    pub data_model: Value,
    /// The catalog: `insight`, `studio`, `spatial` (the default) or `forms`.
    #[serde(default)]
    pub catalog: Option<String>,
}

fn catalog_named(name: Option<&str>) -> Result<&'static Catalog, String> {
    match name {
        None | Some("spatial") => Ok(catalogs::spatial()),
        Some("studio") => Ok(catalogs::studio()),
        Some("insight") => Ok(catalogs::insight()),
        Some("forms") => Ok(Catalog::forms()),
        Some(other) => Err(format!(
            "there is no catalog {other}; use insight, studio, spatial or forms"
        )),
    }
}

/// The `show_drawing` tool as `tools/list` lists it.
pub fn drawing_tool() -> Tool {
    let schema: JsonObject = serde_json::from_value(json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["components"],
        "properties": {
            "title": { "type": "string", "maxLength": 160, "description": "The title above it." },
            "components": {
                "type": "array", "minItems": 1, "items": { "type": "object" },
                "description": "A2UI components, one with id root, in 1.0's syntax or v0.9.1's."
            },
            "dataModel": { "type": "object", "description": "The data the components bind." },
            "catalog": { "type": "string", "enum": ["insight", "studio", "spatial", "forms"],
                         "default": "spatial" },
        },
    }))
    .expect("the input schema is an object");
    let mut tool = Tool::new(
        DRAWING_TOOL,
        drawing_description(),
        std::sync::Arc::new(schema),
    )
    .annotate(
        ToolAnnotations::new()
            .read_only(true)
            .destructive(false)
            .idempotent(true)
            .open_world(false),
    );
    tool.title = Some("Show a drawing".into());
    tool.meta = Some(meta(json!({
        "ui": { "resourceUri": DRAWING_URI, "visibility": ["model"] }
    })));
    tool
}

fn drawing_description() -> String {
    let extensions: Vec<&str> = catalogs::spatial().extensions().collect();
    format!(
        "Shows the person a drawing, read-only, in this conversation: A2UI components of \
         adk-ui's catalogs, the forms components (Text, Row, Column, List, Card, Tabs, Divider \
         and fields shown with their values) and {}. Use it for dashboards, reports and \
         summaries. Bind data with {{\"@path\": \"/rows\"}} into dataModel. It draws no buttons: \
         ask the person in words for anything they must send. Never show a decision's words \
         such as Allow once or Approve, a secret or a link: such a drawing is refused.",
        extensions.join(", ")
    )
}

/// The page as `resources/read` returns it.
pub fn drawing_page() -> ResourceContents {
    ResourceContents::text(DRAWING_PAGE, DRAWING_URI)
        .with_mime_type(APP_MIME_TYPE)
        .with_meta(meta(json!({ "ui": { "prefersBorder": true } })))
}

/// Runs one `show_drawing` call: the drawing's words, and its resolved tree as structured
/// content for the page; or why it was refused.
pub fn show_drawing(arguments: Option<JsonObject>) -> CallToolResult {
    let input: DrawingInput =
        match serde_json::from_value(Value::Object(arguments.unwrap_or_default())) {
            Ok(input) => input,
            Err(error) => return refused(&format!("its arguments don't fit: {error}")),
        };
    let catalog = match catalog_named(input.catalog.as_deref()) {
        Ok(catalog) => catalog,
        Err(reason) => return refused(&reason),
    };
    let components = match components_in(input.components, catalog) {
        Ok(components) => components,
        Err(error) => return refused(&format!("components{error}")),
    };
    let draft = SurfaceDraft {
        surface_id: "drawing".into(),
        kind: DrawnKind::Screen,
        title: input.title.filter(|title| !title.trim().is_empty()),
        components,
        data_model: if input.data_model.is_null() {
            json!({})
        } else {
            input.data_model
        },
        actions: Vec::new(),
    };
    let validated = match Validator::new(catalog).validate(&draft) {
        Ok(validated) => validated,
        Err(refusal) => return refused(&refusal.technical()),
    };
    let tree = drawn_tree(catalog, &validated);
    let title = validated.draft.title.clone();
    let mut text = title
        .clone()
        .map(|title| format!("{title}\n"))
        .unwrap_or_default();
    outline(&tree, &mut text);
    if text.chars().count() > TEXT_CHARS {
        text = text.chars().take(TEXT_CHARS - 1).collect::<String>() + "…";
    }
    let mut result = CallToolResult::success(vec![ContentBlock::text(text)]);
    result.structured_content = Some(json!({ "title": title, "tree": tree }));
    result
}

/// The drawing's words, one line each: what a host without MCP Apps shows.
fn outline(node: &DrawnNode, out: &mut String) {
    if let Some(chart) = &node.chart {
        let _ = writeln!(out, "{}", chart.summary);
        return;
    }
    if let Some(scene) = &node.scene {
        let _ = writeln!(out, "{}", scene.summary);
        return;
    }
    if let Some(field) = &node.field {
        let label = node.text.as_deref().unwrap_or(&field.path);
        let value = match &field.value {
            Value::String(text) => text.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        let _ = writeln!(out, "{label}: {value}");
        return;
    }
    if let Some(text) = node.text.as_deref().filter(|text| !text.is_empty()) {
        let _ = writeln!(out, "{text}");
    }
    for child in &node.children {
        outline(child, out);
    }
}

fn refused(reason: &str) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(format!(
        "Not shown: {reason}. Change the drawing and call {DRAWING_TOOL} again."
    ))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use adk_surface::genui::reserved;

    fn call(arguments: Value) -> CallToolResult {
        show_drawing(Some(serde_json::from_value(arguments).unwrap()))
    }

    #[test]
    fn a_drawing_is_checked_resolved_and_shown_as_its_tree() {
        let shown = call(json!({
            "title": "Revenue",
            "catalog": "insight",
            "components": [
                { "id": "root", "component": "Column", "children": ["total", "chart"] },
                { "id": "total", "component": "Metric", "label": "This quarter", "value": { "@path": "/total" } },
                { "id": "chart", "component": "Chart", "kind": "bar", "data": { "@path": "/rows" },
                  "x": { "key": "q" }, "series": [{ "key": "v", "label": "Revenue" }] }
            ],
            "dataModel": { "total": 1200, "rows": [{ "q": "Q1", "v": 4 }, { "q": "Q2", "v": 7 }] }
        }));
        assert_ne!(shown.is_error, Some(true), "{:?}", shown.content);
        let structured = shown.structured_content.unwrap();
        assert_eq!(structured["title"], "Revenue");
        assert_eq!(structured["tree"]["component"], "Column");
        let text = serde_json::to_string(&shown.content).unwrap();
        assert!(text.contains("This quarter"), "{text}");
    }

    #[test]
    fn a_button_a_decision_word_or_a_link_is_refused() {
        for (component, says) in [
            (
                json!({ "id": "root", "component": "Button", "child": "l", "action": { "event": { "name": "go" } } }),
                "action",
            ),
            (
                json!({ "id": "root", "component": "Text", "text": "Allow once" }),
                "Allow once",
            ),
            (
                json!({ "id": "root", "component": "Text", "text": "see https://example.com" }),
                "link",
            ),
        ] {
            let refused = call(
                json!({ "components": [component.clone(), { "id": "l", "component": "Text", "text": "Go" }] }),
            );
            assert_eq!(refused.is_error, Some(true), "{component}");
            let text = serde_json::to_string(&refused.content).unwrap();
            assert!(
                text.to_lowercase().contains(&says.to_lowercase()),
                "{component}: {text}"
            );
        }
        let unknown = call(json!({ "catalog": "nope", "components": [] }));
        assert_eq!(unknown.is_error, Some(true));
    }

    #[test]
    fn the_page_shows_no_decision_words_and_reaches_no_site() {
        for line in DRAWING_PAGE.lines() {
            assert_eq!(reserved(line), None, "{line}");
        }
        assert!(!DRAWING_PAGE.contains("http://") && !DRAWING_PAGE.contains("https://"));
        assert!(
            !DRAWING_PAGE.contains("innerHTML"),
            "the page sets text only"
        );
        let tool = drawing_tool();
        assert_eq!(
            tool.meta.as_ref().unwrap().0["ui"]["resourceUri"],
            DRAWING_URI
        );
        assert_eq!(
            reserved(&drawing_description()),
            Some("Allow once"),
            "it names what never to show"
        );
    }
}
