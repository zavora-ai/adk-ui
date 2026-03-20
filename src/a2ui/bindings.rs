use serde::{Deserialize, Serialize};

/// A2UI-friendly dynamic string binding.
///
/// - `Literal` embeds a fixed string.
/// - `Path` references a data model path (e.g. "/user/name").
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DynamicString {
    #[serde(rename = "literalString")]
    Literal(String),
    #[serde(rename = "path")]
    Path(String),
}

impl DynamicString {
    pub fn literal(value: impl Into<String>) -> Self {
        Self::Literal(value.into())
    }

    pub fn path(value: impl Into<String>) -> Self {
        Self::Path(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_literal_string() {
        let value = DynamicString::literal("hello");
        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(serialized, json!({ "literalString": "hello" }));
    }

    #[test]
    fn serializes_path_string() {
        let value = DynamicString::path("/user/name");
        let serialized = serde_json::to_value(&value).unwrap();
        assert_eq!(serialized, json!({ "path": "/user/name" }));
    }
}
