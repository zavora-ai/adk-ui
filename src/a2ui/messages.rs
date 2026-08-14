//! A2UI agent→renderer and renderer→agent message envelopes (v0.9 / v0.9.1).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Wire protocol version written on every A2UI envelope.
pub const A2UI_PROTOCOL_VERSION: &str = "v0.9.1";

/// Official MIME type for A2UI payloads (v0.9.1 standardization).
pub const A2UI_MIME_TYPE: &str = "application/a2ui+json";

/// Legacy MIME type still accepted by some clients.
pub const A2UI_MIME_TYPE_LEGACY: &str = "application/json+a2ui";

/// Basic catalog id used by A2UI v0.9.1 examples (string identifier, not a download).
pub const A2UI_BASIC_CATALOG_ID: &str =
    "https://a2ui.org/specification/v0_9_1/catalogs/basic/catalog.json";

/// Basic catalog id for the A2UI v1.0 release candidate.
pub const A2UI_BASIC_CATALOG_ID_V1: &str =
    "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";

fn default_a2ui_version() -> String {
    A2UI_PROTOCOL_VERSION.to_string()
}

/// A2UI createSurface payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSurface {
    pub surface_id: String,
    pub catalog_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_data_model: Option<bool>,
    /// Optional bulk components (A2UI v1.0 candidate; ignored by pure v0.9 clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Value>>,
    /// Optional initial data model (A2UI v1.0 candidate).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_model: Option<Value>,
}

/// A2UI updateComponents payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponents {
    pub surface_id: String,
    pub components: Vec<Value>,
}

/// A2UI updateDataModel payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDataModel {
    pub surface_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
}

/// A2UI deleteSurface payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: String,
}

/// Envelope: createSurface message with protocol version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSurfaceMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    #[serde(rename = "createSurface")]
    pub create_surface: CreateSurface,
}

/// Envelope: updateComponents message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComponentsMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    #[serde(rename = "updateComponents")]
    pub update_components: UpdateComponents,
}

/// Envelope: updateDataModel message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDataModelMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    #[serde(rename = "updateDataModel")]
    pub update_data_model: UpdateDataModel,
}

/// Envelope: deleteSurface message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSurfaceMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    #[serde(rename = "deleteSurface")]
    pub delete_surface: DeleteSurface,
}

/// Agent→renderer response to a renderer action that set `wantResponse: true` (A2UI v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponseBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<A2uiActionResponseError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiActionResponseError {
    pub code: String,
    pub message: String,
}

/// Envelope: actionResponse message (A2UI v1.0).
///
/// Wire shape:
/// `{ "version": "v1.0", "actionId": "...", "actionResponse": { "value" | "error" } }`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponseMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    pub action_id: String,
    pub action_response: ActionResponseBody,
}

impl ActionResponseMessage {
    pub fn success(action_id: impl Into<String>, value: Value) -> Self {
        Self {
            version: "v1.0".to_string(),
            action_id: action_id.into(),
            action_response: ActionResponseBody {
                value: Some(value),
                error: None,
            },
        }
    }

    pub fn failure(
        action_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            version: "v1.0".to_string(),
            action_id: action_id.into(),
            action_response: ActionResponseBody {
                value: None,
                error: Some(A2uiActionResponseError {
                    code: code.into(),
                    message: message.into(),
                }),
            },
        }
    }
}

/// Agent-initiated function call (A2UI v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFunctionBody {
    pub call: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFunctionMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    pub function_call_id: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub want_response: bool,
    pub call_function: CallFunctionBody,
}

/// Renderer→agent function result (A2UI v1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseBody {
    pub function_call_id: String,
    pub call: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionResponseMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    pub function_response: FunctionResponseBody,
}

/// A2UI agent→renderer message envelope (exactly one of the variants).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum A2uiMessage {
    CreateSurface(CreateSurfaceMessage),
    UpdateComponents(UpdateComponentsMessage),
    UpdateDataModel(UpdateDataModelMessage),
    DeleteSurface(DeleteSurfaceMessage),
    ActionResponse(ActionResponseMessage),
    CallFunction(CallFunctionMessage),
}

impl A2uiMessage {
    pub fn version(&self) -> &str {
        match self {
            Self::CreateSurface(m) => &m.version,
            Self::UpdateComponents(m) => &m.version,
            Self::UpdateDataModel(m) => &m.version,
            Self::DeleteSurface(m) => &m.version,
            Self::ActionResponse(m) => &m.version,
            Self::CallFunction(m) => &m.version,
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        let version = version.into();
        match &mut self {
            Self::CreateSurface(m) => m.version = version,
            Self::UpdateComponents(m) => m.version = version,
            Self::UpdateDataModel(m) => m.version = version,
            Self::DeleteSurface(m) => m.version = version,
            Self::ActionResponse(m) => m.version = version,
            Self::CallFunction(m) => m.version = version,
        }
        self
    }
}

/// Convenience constructors that stamp the current production version.
impl CreateSurfaceMessage {
    pub fn new(create_surface: CreateSurface) -> Self {
        Self {
            version: default_a2ui_version(),
            create_surface,
        }
    }
}

impl UpdateComponentsMessage {
    pub fn new(update_components: UpdateComponents) -> Self {
        Self {
            version: default_a2ui_version(),
            update_components,
        }
    }
}

impl UpdateDataModelMessage {
    pub fn new(update_data_model: UpdateDataModel) -> Self {
        Self {
            version: default_a2ui_version(),
            update_data_model,
        }
    }
}

impl DeleteSurfaceMessage {
    pub fn new(delete_surface: DeleteSurface) -> Self {
        Self {
            version: default_a2ui_version(),
            delete_surface,
        }
    }
}

// ── Renderer → agent ──────────────────────────────────────────────────

/// Client/renderer action message (button, form submit, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiClientAction {
    pub name: String,
    pub surface_id: String,
    pub source_component_id: String,
    pub timestamp: String,
    pub context: Value,
    /// When true, the renderer expects an `actionResponse` correlated by `action_id` (v1.0).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub want_response: bool,
    /// Optional correlation id when the renderer expects an actionResponse (v1.0 path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
}

/// Envelope for a renderer→agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2uiClientActionMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    pub action: A2uiClientAction,
}

/// Validation / runtime error reported by the renderer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct A2uiValidationFailedError {
    pub code: String,
    pub surface_id: String,
    pub path: String,
    pub message: String,
}

impl A2uiValidationFailedError {
    pub fn new(
        surface_id: impl Into<String>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: "VALIDATION_FAILED".to_string(),
            surface_id: surface_id.into(),
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Envelope for a renderer error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2uiErrorMessage {
    #[serde(default = "default_a2ui_version")]
    pub version: String,
    pub error: A2uiValidationFailedError,
}

/// Renderer→agent message union.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum A2uiClientMessage {
    Action(A2uiClientActionMessage),
    FunctionResponse(FunctionResponseMessage),
    Error(A2uiErrorMessage),
}

/// Write an action response value into a data model using a JSON Pointer path.
pub fn apply_action_response_value(
    data_model: &mut Value,
    response_path: Option<&str>,
    value: Value,
) {
    let path = response_path.unwrap_or("/__a2ui/lastActionResponse");
    if path.is_empty() || path == "/" {
        *data_model = value;
        return;
    }
    let tokens: Vec<String> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|t| !t.is_empty())
        .map(|t| t.replace("~1", "/").replace("~0", "~"))
        .collect();
    if tokens.is_empty() {
        *data_model = value;
        return;
    }

    // Rebuild as a nested object tree, merging into existing data_model.
    let mut leaf = value;
    for token in tokens.iter().rev() {
        leaf = json!({ token: leaf });
    }
    merge_json(data_model, leaf);
}

fn merge_json(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                match target_map.get_mut(&key) {
                    Some(existing) => merge_json(existing, value),
                    None => {
                        target_map.insert(key, value);
                    }
                }
            }
        }
        (target, source) => {
            *target = source;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_surface_includes_version_and_mime_constants() {
        let msg = CreateSurfaceMessage::new(CreateSurface {
            surface_id: "main".into(),
            catalog_id: A2UI_BASIC_CATALOG_ID.into(),
            theme: None,
            send_data_model: Some(true),
            components: None,
            data_model: None,
        });
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(value["version"], "v0.9.1");
        assert_eq!(value["createSurface"]["surfaceId"], "main");
        assert_eq!(A2UI_MIME_TYPE, "application/a2ui+json");
    }

    #[test]
    fn validation_failed_error_uses_standard_code() {
        let err = A2uiValidationFailedError::new("s1", "/components/0/text", "bad type");
        let value = serde_json::to_value(A2uiErrorMessage {
            version: default_a2ui_version(),
            error: err,
        })
        .unwrap();
        assert_eq!(value["error"]["code"], "VALIDATION_FAILED");
        assert_eq!(value["error"]["path"], "/components/0/text");
    }

    #[test]
    fn client_action_round_trips() {
        let msg = A2uiClientActionMessage {
            version: default_a2ui_version(),
            action: A2uiClientAction {
                name: "submit".into(),
                surface_id: "form".into(),
                source_component_id: "btn".into(),
                timestamp: "2026-08-09T00:00:00Z".into(),
                context: json!({"ok": true}),
                want_response: true,
                action_id: Some("act-1".into()),
            },
        };
        let raw = serde_json::to_string(&msg).unwrap();
        let parsed: A2uiClientActionMessage = serde_json::from_str(&raw).unwrap();
        assert_eq!(parsed.action.name, "submit");
        assert_eq!(parsed.action.action_id.as_deref(), Some("act-1"));
        assert!(parsed.action.want_response);
        assert!(!raw.contains("responsePath"));
    }

    #[test]
    fn action_response_success_serializes_v1() {
        let msg = ActionResponseMessage::success("act-1", json!(["apple", "application"]));
        let value = serde_json::to_value(&msg).unwrap();
        assert_eq!(value["version"], "v1.0");
        assert_eq!(value["actionId"], "act-1");
        assert_eq!(value["actionResponse"]["value"][0], "apple");
    }

    #[test]
    fn apply_action_response_writes_nested_path() {
        let mut model = json!({"form": {"email": "a@b.c"}});
        apply_action_response_value(
            &mut model,
            Some("/form/suggestions"),
            json!(["alice", "alex"]),
        );
        assert_eq!(model["form"]["email"], "a@b.c");
        assert_eq!(model["form"]["suggestions"][0], "alice");
    }
}
