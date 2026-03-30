use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parameters for the render_modal tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderModalParams {
    /// Modal title shown in the header
    pub title: String,
    /// Main message or content to display
    pub message: String,
    /// Modal size: small, medium, large, full
    #[serde(default = "default_size")]
    pub size: String,
    /// Whether to show the close button
    #[serde(default = "default_true")]
    pub closable: bool,
    /// Optional confirm button label
    pub confirm_label: Option<String>,
    /// Optional cancel button label
    pub cancel_label: Option<String>,
    /// Action ID for the confirm button
    #[serde(default = "default_confirm_action")]
    pub confirm_action: String,
    /// Action ID for the cancel button
    #[serde(default = "default_cancel_action")]
    pub cancel_action: String,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

fn default_size() -> String {
    "medium".to_string()
}

fn default_true() -> bool {
    true
}

fn default_confirm_action() -> String {
    "modal_confirm".to_string()
}

fn default_cancel_action() -> String {
    "modal_cancel".to_string()
}

/// Tool for rendering modal dialogs.
///
/// Creates overlay modal dialogs for focused interactions, confirmations,
/// or important messages that require user attention.
///
/// # Example JSON Parameters
///
/// ```json
/// {
///   "title": "Confirm Action",
///   "message": "Are you sure you want to proceed with this action?",
///   "size": "medium",
///   "confirm_label": "Yes, Proceed",
///   "cancel_label": "Cancel",
///   "confirm_action": "confirm_action",
///   "cancel_action": "cancel_action"
/// }
/// ```
pub struct RenderModalTool;

impl RenderModalTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderModalTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderModalTool {
    fn name(&self) -> &str {
        "render_modal"
    }

    fn description(&self) -> &str {
        "Render a modal dialog overlay. Use for important messages, confirmations, or focused interactions that require user attention."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderModalParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderModalParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        let size = match params.size.as_str() {
            "small" => ModalSize::Small,
            "large" => ModalSize::Large,
            "full" => ModalSize::Full,
            _ => ModalSize::Medium,
        };

        // Build modal content
        let content = vec![Component::Text(Text {
            id: None,
            content: params.message,
            variant: TextVariant::Body,
        })];

        // Build footer with buttons if provided
        let footer = if params.confirm_label.is_some() || params.cancel_label.is_some() {
            let mut buttons = Vec::new();
            if let Some(cancel) = params.cancel_label {
                buttons.push(Component::Button(Button {
                    id: None,
                    label: cancel,
                    action_id: params.cancel_action,
                    variant: ButtonVariant::Secondary,
                    disabled: false,
                    icon: None,
                }));
            }
            if let Some(confirm) = params.confirm_label {
                buttons.push(Component::Button(Button {
                    id: None,
                    label: confirm,
                    action_id: params.confirm_action,
                    variant: ButtonVariant::Primary,
                    disabled: false,
                    icon: None,
                }));
            }
            Some(buttons)
        } else {
            None
        };

        let ui = UiResponse::new(vec![Component::Modal(Modal {
            id: None,
            title: params.title,
            content,
            footer,
            size,
            closable: params.closable,
        })]);

        render_ui_response_with_protocol(ui, &protocol_options, "modal")
    }
}
