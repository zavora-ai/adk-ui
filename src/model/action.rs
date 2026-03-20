use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical representation for UI actions emitted by components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalAction {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

impl CanonicalAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            context: None,
        }
    }

    pub fn with_context(mut self, context: Option<Value>) -> Self {
        self.context = context;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_action_serializes_with_optional_context() {
        let action = CanonicalAction::new("submit").with_context(Some(json!({ "k": "v" })));
        let value = serde_json::to_value(action).expect("serialize canonical action");
        assert_eq!(value["name"], "submit");
        assert_eq!(value["context"]["k"], "v");
    }
}
