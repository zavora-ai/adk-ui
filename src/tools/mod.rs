mod protocol_output;
mod render_alert;
mod render_card;
mod render_chart;
mod render_confirm;
mod render_form;
mod render_kit;
mod render_layout;
mod render_modal;
mod render_page;
mod render_progress;
mod render_screen;
mod render_table;
mod render_toast;

pub use render_alert::RenderAlertTool;
pub use render_card::RenderCardTool;
pub use render_chart::RenderChartTool;
pub use render_confirm::RenderConfirmTool;
pub use render_form::RenderFormTool;
pub use render_kit::RenderKitTool;
pub use render_layout::RenderLayoutTool;
pub use render_modal::RenderModalTool;
pub use render_page::RenderPageTool;
pub use render_progress::RenderProgressTool;
pub use render_screen::RenderScreenTool;
pub use render_table::RenderTableTool;
pub use render_toast::RenderToastTool;

pub(crate) use protocol_output::{
    LegacyProtocolOptions, SurfaceProtocolOptions, render_ui_response_with_protocol,
};

use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

/// Generate a Gemini-compatible schema matching the adk-tool 0.4 FunctionTool pattern.
///
/// Serializes the full `RootSchema` (not just `schema.schema`) and strips fields
/// that the Gemini API rejects: `$schema`, `nullable`, `default`, `additionalProperties`,
/// `definitions`, `$ref`, and the root-level `description` (tool description is sent
/// separately in the FunctionDeclaration).
pub(crate) fn generate_gemini_schema<T>() -> Value
where
    T: JsonSchema + Serialize,
{
    let settings = schemars::r#gen::SchemaSettings::openapi3().with(|s| {
        s.inline_subschemas = true;
        s.meta_schema = None;
    });
    let generator = schemars::r#gen::SchemaGenerator::new(settings);
    let mut schema = generator.into_root_schema_for::<T>();
    schema.schema.metadata().title = None;

    // Serialize the full RootSchema (same as adk-tool FunctionTool)
    let mut value = serde_json::to_value(schema).unwrap();
    clean_schema(&mut value);

    // Remove root-level description — the tool description is provided separately
    // in the FunctionDeclaration, not inside the parameters schema.
    if let Value::Object(map) = &mut value {
        map.remove("description");
    }

    value
}

/// Remove fields that the Gemini API does not support.
///
/// The Gemini API uses a subset of OpenAPI 3.0 Schema and rejects:
/// - `nullable` (causes HTTP 400)
/// - `default` (not part of Gemini's Schema spec)
/// - `$schema`, `definitions`, `$ref` (JSON Schema meta-fields)
/// - `additionalProperties` (not supported)
fn clean_schema(value: &mut Value) {
    if let Value::Object(map) = value {
        map.remove("$schema");
        map.remove("definitions");
        map.remove("$ref");
        map.remove("additionalProperties");
        map.remove("nullable");
        map.remove("default");

        // Recursively clean nested objects
        for (_, v) in map.iter_mut() {
            clean_schema(v);
        }
    } else if let Value::Array(arr) = value {
        for v in arr.iter_mut() {
            clean_schema(v);
        }
    }
}
