//! Shared pieces of adk-ui's component schemas: every component is shaped as `forms` 1.0
//! shapes its own (A2UI v0.9.1), closed to properties it does not declare.

use serde_json::{Value, json};

/// The A2UI v0.9.1 common types.
pub(crate) const COMMON: &str = "https://a2ui.org/specification/v0_9/common_types.json#/$defs";

/// A component's schema: the A2UI common component properties, the catalog's own common
/// properties, then `properties`, with `required` besides `component`; nothing undeclared.
pub(crate) fn component(name: &str, mut properties: Value, required: &[&str]) -> Value {
    properties["component"] = json!({ "const": name });
    let mut required_all = vec!["component"];
    required_all.extend_from_slice(required);
    json!({
        "type": "object",
        "allOf": [
            { "$ref": format!("{COMMON}/ComponentCommon") },
            { "$ref": "#/$defs/CatalogComponentCommon" },
            { "type": "object", "properties": properties, "required": required_all }
        ],
        "unevaluatedProperties": false
    })
}

/// Text: a literal, a binding, or `formatString` / `pluralize`.
pub(crate) fn text() -> Value {
    json!({ "$ref": format!("{COMMON}/DynamicString") })
}

/// A number: a literal or a binding.
pub(crate) fn number() -> Value {
    json!({ "$ref": format!("{COMMON}/DynamicNumber") })
}

/// Any value: a literal or a binding.
pub(crate) fn value() -> Value {
    json!({ "$ref": format!("{COMMON}/DynamicValue") })
}

/// Data: a binding into the data model, or a literal list of `items`.
pub(crate) fn data(items: Value, max: usize) -> Value {
    json!({
        "oneOf": [
            { "$ref": format!("{COMMON}/DataBinding") },
            { "type": "array", "items": items, "maxItems": max }
        ]
    })
}

/// One of `values`.
pub(crate) fn one_of(values: &[&str]) -> Value {
    json!({ "type": "string", "enum": values })
}

/// A data key: the name of a field of each row.
pub(crate) fn key() -> Value {
    json!({ "type": "string", "minLength": 1, "maxLength": 64 })
}

/// The tones a component may take; the host draws each in its own colours, never a
/// decision's.
pub(crate) const TONES: &[&str] = &["neutral", "info", "success", "warning", "danger"];
