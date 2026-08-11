use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::schema::UiEvent;

/// A2UI v0.9 action event payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiActionEvent {
    pub name: String,
    pub surface_id: String,
    pub source_component_id: String,
    pub timestamp: String,
    pub context: Value,
}

#[derive(Debug, Clone)]
pub struct A2uiActionMetadata {
    pub surface_id: String,
    pub source_component_id: String,
    pub timestamp: DateTime<Utc>,
}

impl A2uiActionMetadata {
    pub fn new(surface_id: impl Into<String>, source_component_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            source_component_id: source_component_id.into(),
            timestamp: Utc::now(),
        }
    }
}

/// Maps ADK UiEvent into A2UI action events.
#[derive(Debug, Default)]
pub struct UiEventMapper;

impl UiEventMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn to_a2ui_action(&self, event: &UiEvent, meta: &A2uiActionMetadata) -> A2uiActionEvent {
        let (name, context) = match event {
            UiEvent::FormSubmit { action_id, data } => (
                action_id.clone(),
                serde_json::to_value(data).unwrap_or_else(|_| Value::Object(Default::default())),
            ),
            UiEvent::ButtonClick { action_id } => {
                (action_id.clone(), Value::Object(Default::default()))
            }
            UiEvent::InputChange { name, value } => (
                "input_change".to_string(),
                serde_json::json!({ "name": name, "value": value }),
            ),
            UiEvent::TabChange { index } => (
                "tab_change".to_string(),
                serde_json::json!({ "index": index }),
            ),
        };

        A2uiActionEvent {
            name,
            surface_id: meta.surface_id.clone(),
            source_component_id: meta.source_component_id.clone(),
            timestamp: meta.timestamp.to_rfc3339(),
            context,
        }
    }

    /// Map into a versioned client action envelope (A2UI v0.9.1 / v1.0 wire shape).
    pub fn to_client_action_message(
        &self,
        event: &UiEvent,
        meta: &A2uiActionMetadata,
        action_id: Option<String>,
    ) -> crate::a2ui::messages::A2uiClientActionMessage {
        self.to_client_action_message_v1(event, meta, action_id, false, None)
    }

    /// Map into a v1.0 client action that may request an `actionResponse`.
    pub fn to_client_action_message_v1(
        &self,
        event: &UiEvent,
        meta: &A2uiActionMetadata,
        action_id: Option<String>,
        want_response: bool,
        response_path: Option<String>,
    ) -> crate::a2ui::messages::A2uiClientActionMessage {
        // responsePath is renderer-local metadata and is not part of the action wire message.
        let _ = response_path;
        let action = self.to_a2ui_action(event, meta);
        let version = if want_response {
            "v1.0".to_string()
        } else {
            crate::a2ui::messages::A2UI_PROTOCOL_VERSION.to_string()
        };
        crate::a2ui::messages::A2uiClientActionMessage {
            version,
            action: crate::a2ui::messages::A2uiClientAction {
                name: action.name,
                surface_id: action.surface_id,
                source_component_id: action.source_component_id,
                timestamp: action.timestamp,
                context: action.context,
                want_response,
                action_id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::UiEvent;
    use serde_json::json;

    #[test]
    fn maps_form_submit_with_context() {
        let mapper = UiEventMapper::new();
        let event = UiEvent::FormSubmit {
            action_id: "submit".to_string(),
            data: [("email".to_string(), json!("a@b.com"))]
                .into_iter()
                .collect(),
        };
        let meta = A2uiActionMetadata::new("main", "form-1");
        let mapped = mapper.to_a2ui_action(&event, &meta);
        assert_eq!(mapped.name, "submit");
        assert_eq!(mapped.surface_id, "main");
        assert_eq!(mapped.source_component_id, "form-1");
        assert!(mapped.context.get("email").is_some());
    }
}
