use crate::compat::{Result, Tool, ToolContext};
use crate::schema::*;
use crate::tools::{LegacyProtocolOptions, render_ui_response_with_protocol};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// Parameters for the render_alert tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RenderAlertParams {
    /// Alert title
    pub title: String,
    /// Optional detailed message
    #[serde(default)]
    pub description: Option<String>,
    /// Alert variant: info, success, warning, error
    #[serde(default = "default_variant")]
    pub variant: String,
    /// Optional protocol output configuration.
    #[serde(flatten)]
    pub protocol: LegacyProtocolOptions,
}

fn default_variant() -> String {
    "info".to_string()
}

/// Tool for rendering alerts/notifications
pub struct RenderAlertTool;

impl RenderAlertTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RenderAlertTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for RenderAlertTool {
    fn name(&self) -> &str {
        "render_alert"
    }

    fn description(&self) -> &str {
        "Render an alert notification to inform the user about something. Use for success messages, warnings, errors, or important information."
    }

    fn parameters_schema(&self) -> Option<Value> {
        Some(super::generate_gemini_schema::<RenderAlertParams>())
    }

    async fn execute(&self, _ctx: Arc<dyn ToolContext>, args: Value) -> Result<Value> {
        let params: RenderAlertParams = serde_json::from_value(args)
            .map_err(|e| crate::compat::AdkError::Tool(format!("Invalid parameters: {}", e)))?;
        let protocol_options = params.protocol.clone();

        let variant = match params.variant.as_str() {
            "success" => AlertVariant::Success,
            "warning" => AlertVariant::Warning,
            "error" => AlertVariant::Error,
            _ => AlertVariant::Info,
        };

        let ui = UiResponse::new(vec![Component::Alert(Alert {
            id: None,
            title: params.title,
            description: params.description,
            variant,
        })]);

        render_ui_response_with_protocol(ui, &protocol_options, "alert")
    }
}
